import {Contract, WalletConnection} from "near-api-js";
import {Config, getConfig} from "../config";
import * as nearAPI from "near-api-js";

declare type TokenId = string;
declare type Balance = string;

interface User{
    accountId: string;
    balanceNear: Balance;
    balanceXdho: Balance;
    balanceOtherTokens: Map<TokenId, Balance>
}

interface MyContract{
    get_tokens(): Promise<any>;
    get_balance(args: {owner_id:string, token_id: string}): Promise<string>;
}

export interface NearContext{
    contract: Contract & MyContract,
    currentUser: User,
    config: Config,
    wallet: WalletConnection,
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

    // Load in account data
    let currentUser;
    if(wallet.getAccountId()) {
        currentUser = {
            accountId: wallet.getAccountId(),
            balance: (await wallet.account().state()).amount
        };
    }

    // Initializing our contract APIs by contract name and configuration
    const contract: Contract = await new nearAPI.Contract(
        wallet.account(),
        config.contractName, {
            // View methods are read-only – they don't modify the state, but usually return some value
            viewMethods: ['get_tokens', "get_balance"],
            // Change methods can modify the state, but you don't receive the returned value when called
            changeMethods: []
        });

    // @ts-ignore
    return { contract, currentUser, config, wallet };
}

export interface Store{
    nearContext: NearContext,
}