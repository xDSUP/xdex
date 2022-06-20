import * as nearAPI from "near-api-js";
import {Contract, WalletConnection} from "near-api-js";
import {Config, getConfig} from "../config";
import {action, makeObservable, observable, runInAction} from "mobx";
import {f64, i32, u64} from "./helper";
import Big from "big.js";
import BN from "bn.js";

export declare type TokenId = string;
export declare type AccountId = string;
export declare type RequestId = string;
export declare type Balance = number;
export declare type Timestamp = u64;
export declare type Side = "Ask" | "Bid";

export const STANDARD_TOKEN = "XDHO";
export const BOATLOAD_OF_GAS = Big(10).times(10 ** 13).toFixed();
export const YOKTO_NEAR = Big(10).times(10 ** 23);

interface User {
    accountId: string;
    balanceNear: Balance;
    balanceXdho: Balance;
    balanceOtherTokens: Map<TokenId, Balance>
}

export interface TokenMetadata {
    title?: string
    description?: string,
    icon?: string,
}


export interface Token {
    token_id: TokenId,
    supply: Balance,
    owner_id: AccountId,
    meta?: TokenMetadata
}

export interface Order {
    order_id: u64,
    order_asset: string,
    price_asset: string,
    type: string,
    side: Side,
    price: f64,
    qty: u64,
    order_creator: String,
}

export interface OrderIndex {
    id: u64,
    price: f64,
    quantity: u64,
    timestamp: u64,
    order_side: Side,
}

export interface StakeInfo {
    staked: Balance,
    created_time: Timestamp
}

declare type RequestStatus = "REJECTED" | "APPROVED" | "PENDING" | "LAUNCHED" | "TRADED";

export interface TokenRequest {
    id: RequestId,
    token_id: String,
    title: String,
    description: String,

    price: Balance,
    supply: Balance,

    status: RequestStatus,
    created_time: Timestamp,
    owner_id: AccountId,
    hash: String,
    votes?: Vote[]
}

export interface LaunchPad {
    price: Balance,
    sell_supply: Balance,
    launched_time: Timestamp,
    token: Token
}

interface UserRequest {
    token_id: String,
    title: String,
    description: String,

    price: Balance,
    supply: Balance,

    hash: String,
}

interface Vote {
    owner_id: AccountId,
    result: boolean
}

export interface Ok {
    Ok: any;
}

export function isOk(object: any): object is Ok {
    return 'Ok' in object;
}

export interface Err {
    Err: any;
}

interface MyContract {
    // view
    get_tokens(): Promise<Token[]>;

    get_balance(args: { owner_id: AccountId, token_id: TokenId }): Promise<Balance>;

    get_balances(args: { owner_id: AccountId, token_ids: TokenId[] }): Promise<Map<TokenId, Balance>>;

    get_orders(args: { account_id: AccountId, token_id: TokenId, side: Side }): Promise<Order[]>;

    get_ask_orders(args: { token_id: TokenId }): Promise<Order[]>;

    get_bid_orders(args: { token_id: TokenId }): Promise<Order[]>;

    get_current_spread(args: { token_id: TokenId }): Promise<Balance[]>;

    get_staking(args: { owner_id: AccountId }): Promise<StakeInfo>;

    get_all_staked(): Promise<Balance>;

    get_count_stakers(): Promise<i32>;

    get_all_requests(): Promise<TokenRequest[]>;

    get_request(args: { request_id: RequestId }): Promise<TokenRequest>;

    is_voting(args: { voter_id: AccountId, request_id: RequestId }): Promise<boolean>;

    get_launchpad_tokens(): Promise<LaunchPad[]>;

    get_launchpad(args: { token_id: TokenId }): Promise<LaunchPad>;

    get_all_votes(args: { request_id: RequestId }): Promise<Vote[]>;

    // change
    new_limit_order(args: { token_id: TokenId, price: f64, quantity: u64, side: Side }, gas: string): Promise<Array<Ok | Err>>;

    new_market_order(args: { token_id: TokenId, quantity: u64, side: Side }, gas: string): Promise<Array<Ok | Err>>;

    cancel_limit_order(args: { token_id: TokenId, id: u64, side: Side }, gas: string): Promise<Array<Ok | Err>>;

    pay_standard_token(args: { amount: string, to: AccountId }, attachedDeposit: BN, gas: string): Promise<any>;

    transfer(args: { new_owner_id: AccountId, token_id: TokenId, amount: u64 }, gas: string): Promise<any>;

    stake(args: { amount: u64 }, gas: string): Promise<any>;

    unstake(gas: string): Promise<any>;

    add_new_request(args: { request: UserRequest }, gas: string): Promise<any>;

    vote(args: { request_id: RequestId, vote: boolean }, gas: string): Promise<any>;

    finalize_request(args: { request_id: RequestId }, gas: string): Promise<any>;

    start_launchpad(args: { request_id: RequestId, launched_time: Timestamp }, gas: string): Promise<any>;

    buy_tokens_on_launchpad(args: { token_id: TokenId, amount: Balance }, gas: string): Promise<any>;

    finalize_my_launchpad(args: { token_id: TokenId }, gas: string): Promise<any>;
}

export class NearContext {
    contract: Contract & MyContract;
    config: Config;

    @observable
    currentUser?: User;
    wallet: WalletConnection;

    @observable
    tokens: Token[];
    @observable
    tokensMap: Map<TokenId, Token>;

    constructor(contract: Contract & MyContract, config: Config, wallet: WalletConnection) {
        makeObservable(this);
        this.contract = contract;
        this.wallet = wallet;
        this.config = config;

        this.tokens = [];
        this.tokensMap = new Map<TokenId, Token>();

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
        return this.wallet.account().state().then(value => runInAction(() => {
            if (this.currentUser) {
                let amount = Number(value.amount.substring(0, value.amount.length - 22));
                this.currentUser.balanceNear = (amount / 100).valueOf();
            }
        }))
    }

    @action.bound
    updateXdhoBalance() {
        if (!this.currentUser)
            return Promise.reject(null);
        return this.contract.get_balance({owner_id: this.currentUser.accountId, token_id: STANDARD_TOKEN})
            .then(value => runInAction(() => {
                if (this.currentUser)
                    this.currentUser.balanceXdho = value;
            }));
    }

    @action.bound
    updateTokens() {
        return this.contract.get_tokens().then(value => runInAction(() => {
            this.tokens = value;
            let newTokensMap = new Map<TokenId, Token>();

            for (const tokenMeta of value)
                newTokensMap.set(tokenMeta.token_id, tokenMeta);
            this.tokensMap = newTokensMap;
        }));
    }

    @action.bound
    updateAllBalance() {
        if (this.currentUser && this.tokens) {
            let allTokens = this.tokens.map(value => value.token_id);
            this.contract.get_balances({owner_id: this.currentUser.accountId, token_ids: allTokens})
                .then(value => runInAction(() => {
                    if (this.currentUser) {
                        this.currentUser.balanceOtherTokens = new Map(value);
                    }
                }));
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
                "get_orders",
                "get_staking",
                "get_all_staked",
                "get_count_stakers",
                "get_all_requests",
                "get_request",
                "is_voting",
                "get_launchpad_tokens",
                "get_launchpad",
                "get_all_votes",
            ],

            // Change methods can modify the state, but you don't receive the returned value when called
            changeMethods: [
                'new_limit_order',
                'new_market_order',
                'cancel_limit_order',
                "transfer",
                "stake",
                "unstake",
                "add_new_request",
                "vote",
                "finalize_request",
                "start_launchpad",
                "buy_tokens_on_launchpad",
                "finalize_my_launchpad",
                "pay_standard_token"
            ]
        });

    // @ts-ignore
    return new NearContext(contract, config, wallet);
}

