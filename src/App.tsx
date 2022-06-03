import React from 'react';
import logo from './images/logo.svg';
import {inject, observer} from "mobx-react";
import {action, makeObservable, observable, runInAction} from "mobx";
import {NearContext, Store} from "./contract/contract";
import {BrowserRouter, Routes, Route, Link, NavLink} from "react-router-dom";
import {InvestorPage} from "./pages/InvestorPage";
import {RegistartorPage} from "./pages/RegistartorPage";
import {EmitentPage} from "./pages/EmitentPage";


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
class PageHeader extends React.Component<{nearContext?: NearContext}> {
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

    render() {
        let nearContext = this.props.nearContext;
        let currentUser = nearContext?.currentUser;
        return <header>
            <div className={"app-header"}>
                <div>
                    <img className={"logo"} src={logo}/>
                    <Link className={"app-name"} to={"/"}>xDEx</Link>
                </div>
                <div className={"nav"}>
                    <NavLink className={""} to={"/investor"}>Торги</NavLink>
                    <NavLink className={""} to={"/registrator"}>Голосование</NavLink>
                    <NavLink className={""} to={"/emitent"}>Эмитенту</NavLink>
                </div>
                <div className={"user-info"}>
                    {currentUser && <>
                        <span className={"xdho-symbol white-section"}>245</span>
                        <div className={"info white-section"}>
                            <span className={"near-symbol"}>23145</span>
                            <span className={"wallet"}>ME...0123124rRe</span>
                        </div>
                        {currentUser
                            ? <button onClick={this.signOut}>Log out</button>
                            : <button onClick={this.signIn}>Log in</button>
                        }
                        </>
                    }
                </div>
            </div>
        </header>;
    }
}

class PageFooter extends React.Component {
    render() {
        return null;
    }
}

@inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))
@observer
class App extends React.Component<{}>{
    render(){

        return <main>
            <div className="App">
                <PageHeader/>
                <Routes>
                    <Route path={"/"} element={<div>Hello1</div>}/>
                    <Route path={"/investor"} element={<InvestorPage/>}/>
                    <Route path={"/registrator"} element={<RegistartorPage/>}/>
                    <Route path={"/emitent"} element={<EmitentPage/>}/>
                </Routes>
                <PageFooter/>
            </div>
        </main>
    }
}

export default App;
