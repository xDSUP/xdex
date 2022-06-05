import React from 'react';
import {inject, observer} from "mobx-react";
import {NearContext, Store} from "./contract/contract";
import {Link, NavLink, Route, Routes} from "react-router-dom";
import {InvestorPage, InvestorPageState} from "./pages/InvestorPage";
import {RegistartorPage} from "./pages/RegistartorPage";
import {EmitentPage} from "./pages/EmitentPage";

import logo from './images/logo.svg';
import wallet from "./images/wallet.svg"

import 'primereact/resources/primereact.css';
import 'primeicons/primeicons.css';
import 'primeflex/primeflex.css';

const PageHeader = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { nearContext?: NearContext }) => {
    var signIn = () => {
        props.nearContext?.wallet.requestSignIn(
            {contractId: props.nearContext.config.contractName, methodNames: []},
            "NEAR hello!"
        );
    };

    var signOut = () => {
        props.nearContext?.wallet.signOut();
        window.location.replace(window.location.origin + window.location.pathname);
    };

    let nearContext = props.nearContext;
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
                    <span className={"xdho-symbol white-section"}>{currentUser.balanceXdho}</span>
                    <div className={"info white-section"}>
                        <span className={"near-symbol"}>{currentUser.balanceNear}</span>
                        <span className={"wallet"}>{currentUser.accountId}</span>
                    </div>
                </>
                }
                {currentUser
                    ? <button onClick={signOut}>Выйти</button>
                    : <button className={"wallet"} onClick={signIn}>
                        <svg width={"24px"} height={"24px"}>
                            <image width={"24px"} height={"24px"} href={wallet}/>
                        </svg>
                        <span>Присоединить кошелёк</span>
                    </button>
                }
            </div>
        </div>
    </header>;

}));

class PageFooter extends React.Component {
    render() {
        return null;
    }
}

@inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))
@observer
class App extends React.Component<{}> {
    investorPageState = new InvestorPageState();

    render() {
        return <main>
            <div className="App layout-theme-light">
                <PageHeader/>
                <div className={"layout-main-container"}>
                    <div className="layout-main">
                        <Routes>
                            <Route path={"/"} element={<div>Hello1</div>}/>
                            <Route path={"/investor"} element={<InvestorPage state={this.investorPageState}/>}/>
                            <Route path={"/registrator"} element={<RegistartorPage/>}/>
                            <Route path={"/emitent"} element={<EmitentPage/>}/>
                        </Routes>
                    </div>
                </div>
                <PageFooter/>
            </div>
        </main>
    }
}

export default App;
