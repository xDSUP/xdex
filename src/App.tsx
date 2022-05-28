import React from 'react';
import logo from './logo.svg';
import './css/App.css';
import {inject, observer} from "mobx-react";
import {action, makeObservable, observable, runInAction} from "mobx";
import {NearContext, Store} from "./contract/contract";


class AppState{
    @observable
    balance: number = 0;

    constructor() {
        makeObservable(this);
    }

}

@inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))
@observer
class App extends React.Component<{nearContext?: NearContext, title?: String}>{

    @action.bound
    updateBalance(){
        let nearContext = this.props.nearContext;
        if(nearContext){
            if (nearContext.currentUser) {
                nearContext.contract
                    .get_balance({owner_id: nearContext.currentUser.accountId, token_id: "XDHO"})
                    // @ts-ignore
                    .then(balance =>
                        {
                            runInAction(() => {
                                this.balance = Number(balance);
                            })
                        });
            }
        }
    }

    @action.bound
    updateBalanceOnce(){
        this.balance++;
    }

    signIn(){
        this.props.nearContext?.wallet.requestSignIn(
            {contractId: this.props.nearContext.config.contractName, methodNames: []},
            "NEAR hello!"
        );
    };

    signOut(){
        this.props.nearContext?.wallet.signOut();
        window.location.replace(window.location.origin + window.location.pathname);
    };

    render(){
        let nearContext = this.props.nearContext;
        let currentUser = nearContext?.currentUser;
        return <main>
            <div className="App">
                <header className="App-header">
                    <img src={logo} className="App-logo" alt="logo" />
                    <p>
                        Edit <code>src/App.tsx</code> and save to reload.
                    </p>
                    <a
                        className="App-link"
                        href="https://reactjs.org"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        {"Learn React with balance " + this.balance}
                    </a>
                    {currentUser ?
                        <p>Currently signed in as: <code>{currentUser.accountId || "hello"}</code></p>
                        :
                        <p>Update or add a status message! Please login to continue.</p>
                    }

                    { currentUser
                        ? <button onClick={() => this.signOut()}>Log out</button>
                        : <button onClick={() => this.signIn()}>Log in</button>
                    }
                    <button onClick={() => runInAction(this.updateBalance)}>update</button>
                    <button onClick={() => this.updateBalanceOnce()}>update</button>
                </header>
            </div>
        </main>
    }
}

export default App;
