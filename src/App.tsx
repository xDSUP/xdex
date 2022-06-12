import React, {useRef} from 'react';
import {inject, observer} from "mobx-react";
import {NearContext} from "./contract/contract";
import {Link, NavLink, Route, Routes} from "react-router-dom";
import {InvestorPage, InvestorPageState} from "./pages/InvestorPage";
import {RegistartorPage, RegistartorPageState} from "./pages/RegistartorPage";
import {EmitentPage, EmitentPageState} from "./pages/EmitentPage";
import logo from './images/logo.svg';
import wallet from "./images/wallet.svg"
import {Store} from "./index";
import {OverlayPanel} from "primereact/overlaypanel";
import {Button} from "primereact/button";


var ProfilePopup = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { nearContext?: NearContext }) => {

    let balances: React.ReactNode[] = [];
    props.nearContext?.currentUser?.balanceOtherTokens.forEach((value, key) => {
        if(value > 0){
            balances.push(<div className={"flex flex-row justify-content-between"}>
                <div className={"flex flex-row align-items-center"}>
                    <i className={"pi pi-money-bill mr-3"}></i>
                    <div className={"flex flex-column"}>
                        <span className={"text-2xl"}>{key}</span>
                        <p className={"text-700"}>{value} {key}</p>
                    </div>
                </div>
                <div>
                    <h5>$12.52</h5>
                </div>
            </div>);
        }
    })

    return <>
        <h2>Счёт</h2>
        <div className={"bid-buttons"}>
            <Button label={"Приобрести XDHO"} className={"p-button mr-4"} icon="pi pi-arrow-down-left"/>
            <Button label={"Продать XDHO"} className={"p-button"} icon="pi pi-arrow-up-right"/>
        </div>
        <h3>Баланс токенов на платформе:</h3>
        <div className={"balances flex flex-column"}>
            {balances}
        </div>
    </>
}));

const PageHeader = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { nearContext?: NearContext }) => {
    const signIn = () => {
        props.nearContext?.wallet.requestSignIn(
            {contractId: props.nearContext.config.contractName, methodNames: []},
            "NEAR hello!"
        );
    };

    const signOut = () => {
        props.nearContext?.wallet.signOut();
        window.location.replace(window.location.origin + window.location.pathname);
    };

    const profilePopup = useRef<OverlayPanel>(null);
    const toggleProfile = (event: any) => {
        profilePopup.current?.toggle(event);
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
                        <span className={"wallet"} onClick={toggleProfile}>{currentUser.accountId}</span>
                        <OverlayPanel ref={profilePopup} appendTo={document.body} showCloseIcon className={"profile-popup"}>
                            <ProfilePopup/>
                        </OverlayPanel>
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

var App = observer(() => {
    let investorPageState = new InvestorPageState();
    let registartorPage = new RegistartorPageState();
    let emitentPage = new EmitentPageState();

    return <main>
        <div className="App layout-theme-light">
            <PageHeader/>
            <div className={"layout-main-container"}>
                <div className="layout-main">
                    <Routes>
                        <Route path={"/"} element={<div>Hello1</div>}/>
                        <Route path={"/investor"} element={<InvestorPage state={investorPageState}/>}/>
                        <Route path={"/registrator"} element={<RegistartorPage state={registartorPage}/>}/>
                        <Route path={"/emitent"} element={<EmitentPage state={emitentPage}/>}/>
                    </Routes>
                </div>
            </div>
            <PageFooter/>
        </div>
    </main>
})

export default App;
