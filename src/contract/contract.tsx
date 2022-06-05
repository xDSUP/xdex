import * as nearAPI from "near-api-js";
import {Contract, WalletConnection} from "near-api-js";
import {Config, getConfig} from "../config";
import {action, makeObservable, observable} from "mobx";
import {f64, u64} from "./helper";

export declare type TokenId = string;
export declare type AccountId = string;
export declare type Balance = number;
export declare type Side = "Ask" | "Bid";

export const STANDARD_TOKEN = "XDHO";

interface User {
    accountId: string;
    balanceNear: Balance;
    balanceXdho: Balance;
    balanceOtherTokens: Map<TokenId, Balance>
}

interface TokenMeta {
    token_id: string,
    supply: number,
    owner_id: AccountId
}

export interface Order{
    order_id: u64,
    order_asset: string,
    price_asset: string,
    type: string,
    side: Side,
    price: f64,
    qty: u64,
    order_creator: String,
}

export interface OrderIndex{
    id: u64,
    price: f64,
    quantity: u64,
    timestamp: u64,
    order_side: Side,
}

interface MyContract {
    // view
    get_tokens(): Promise<TokenMeta[]>;

    get_balance(args: { owner_id: AccountId, token_id: TokenId }): Promise<Balance>;

    get_balances(args: { owner_id: AccountId, token_ids: TokenId[] }): Promise<Map<TokenId, Balance>>;

    get_orders(args: { account_id: AccountId, token_id: TokenId, side: Side}): Promise<Order[]>;

    get_ask_orders(args: { token_id: TokenId }): Promise<OrderIndex[]>;

    get_bid_orders(args: { token_id: TokenId }): Promise<OrderIndex[]>;

    get_current_spread(args: { token_id: TokenId }): Promise<Balance[]>;

    // change
    new_limit_order(args: { owner_id: AccountId, token_id: TokenId }): Promise<Balance>;

    new_market_order(args: { owner_id: AccountId, token_id: TokenId }): Promise<Balance>;

    cancel_limit_order(args: { owner_id: AccountId, token_id: TokenId }): Promise<Balance>;

    transfer(args: { owner_id: AccountId, token_id: TokenId }): Promise<Balance>;
}

export class NearContext {
    contract: Contract & MyContract;
    config: Config;

    @observable
    currentUser?: User;
    wallet: WalletConnection;

    @observable
    tokens: TokenMeta[];

    constructor(contract: Contract & MyContract, config: Config, wallet: WalletConnection) {
        makeObservable(this);
        this.contract = contract;
        this.wallet = wallet;
        this.config = config;

        this.tokens = [];

        if (wallet.getAccountId()) {
            this.currentUser = {
                accountId: wallet.getAccountId(),
                balanceNear: 0,
                balanceXdho: 0,
                balanceOtherTokens: new Map<TokenId, Balance>()
            };
        }

        this.updateTokens().then(value => this.updateAllBalance());
        this.updateNearBalance();
        this.updateXdhoBalance();
    }

    @action.bound
    updateNearBalance() {
        return this.wallet.account().state().then(value => {
            if (this.currentUser) {
                let amount = Number(value.amount.substring(0, value.amount.length - 22));
                this.currentUser.balanceNear = (amount / 100).valueOf();
            }
        })
    }

    @action.bound
    updateXdhoBalance() {
        if (!this.currentUser)
            return Promise.reject(null);
        return this.contract.get_balance({owner_id: this.currentUser.accountId, token_id: STANDARD_TOKEN})
            .then(value => {
                if (this.currentUser)
                    this.currentUser.balanceXdho = value;
            })
    }

    @action.bound
    updateTokens() {
        return this.contract.get_tokens().then(value => {
            this.tokens = value;
        });
    }

    @action.bound
    updateAllBalance() {
        if (this.currentUser && this.tokens) {
            let allTokens = this.tokens.map(value => value.token_id);
            this.contract.get_balances({owner_id: this.currentUser.accountId, token_ids: allTokens})
                .then(value => {
                    if (this.currentUser) {
                        this.currentUser.balanceOtherTokens = new Map(value);
                    }
                })
        }

    }

}

export async function initContract(): Promise<NearContext> {
    const config = getConfig('testnet');

    // Initializing connection to the NEAR TestNet
    const near = await nearAPI.connect({
        headers: {},
        keyStore: new nearAPI.keyStores.BrowserLocalStorageKeyStore(),
        ...config,
    });

    // Needed to access wallet
    const wallet = new nearAPI.WalletConnection(near, null);

    // Initializing our contract APIs by contract name and configuration
    const contract: Contract = await new nearAPI.Contract(
        wallet.account(),
        config.contractName, {
            // View methods are read-only – they don't modify the state, but usually return some value
            viewMethods: [
                'get_tokens',
                "get_balance",
                "get_balances",
                "get_ask_orders",
                "get_bid_orders",
                "get_current_spread",
                "get_orders"
            ],
            // Change methods can modify the state, but you don't receive the returned value when called
            changeMethods: [
                'new_limit_order',
                'new_market_order',
                'cancel_limit_order',
                "transfer"
            ]
        });

    // @ts-ignore
    return new NearContext(contract, config, wallet);
}

export interface Store {
    nearContext: NearContext,
}