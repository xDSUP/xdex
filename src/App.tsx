import React, {useRef, useState} from 'react';
import {inject, observer} from "mobx-react";
import {BOATLOAD_OF_GAS, NearContext, YOKTO_NEAR} from "./contract/contract";
import {Link, NavLink, Route, Routes} from "react-router-dom";
import logo from './images/logo.svg';
import wallet from "./images/wallet.svg"
import {Store, ToastContext} from "./index";
import {OverlayPanel} from "primereact/overlaypanel";
import {Button} from "primereact/button";
import {Dialog} from "primereact/dialog";
import {InvestorPage, InvestorPageState} from "./pages/InvestorPage";
import {EmitentPage, EmitentPageState} from "./pages/EmitentPage";
import {RegistartorPage, RegistartorPageState} from "./pages/RegistartorPage";
import {LaunchpadPage, LaunchpadPageState} from "./pages/LaunchpadPage";
import {Fn} from "ipfs-http-client/dist/src/lib/configure";
import {InputNumber} from "primereact/inputnumber";
import Big from "big.js";
import {utils} from "near-api-js";
import { BN } from './contract/helper';


var ProfilePopup = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext,
    toastContext: allStores.toast as ToastContext,
}))(observer((props: {
    toastContext?: ToastContext,
    nearContext?: NearContext}) => {
    let [dialogSellOpen, setSellDialogOpen] = useState(false);
    let [dialogBuyOpen, setBuyDialogOpen] = useState(false);



    let balances: React.ReactNode[] = [];
    props.nearContext?.currentUser?.balanceOtherTokens.forEach((value, key) => {
        if (value > 0) {
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
        <DialogBuy dialogOpen={dialogBuyOpen} setDialogOpen={setBuyDialogOpen}/>
        <h2>Счёт</h2>
        <div className={"bid-buttons"}>
            <Button label={"Приобрести XDHO"} className={"p-button mr-4"} icon="pi pi-arrow-down-left" onClick={() => setBuyDialogOpen(true)}/>
            <Button label={"Продать XDHO"} className={"p-button"} icon="pi pi-arrow-up-right" onClick={() => setSellDialogOpen(true)}/>
        </div>
        <h3>Баланс токенов на платформе:</h3>
        <div className={"balances flex flex-column"}>
            {balances}
        </div>
    </>
}));

const DialogBuy = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext,
    toastContext: allStores.toast as ToastContext,
}))(observer((props: {
    toastContext?: ToastContext,
    nearContext?: NearContext,
    dialogOpen: boolean,
    setDialogOpen: (arg0: boolean) => void,
}) => {
    const [loading, setLoading] = useState(false);
    const [amount, setAmount] = useState(1);

    let priceStandardTokens = 97;

     const onSubmit = (amount: number) => {
         if (props.nearContext?.currentUser?.accountId) {
             let bal = YOKTO_NEAR.mul(Big(amount));
             props.nearContext?.contract.pay_standard_token({
                 amount: "" + amount, to: props.nearContext?.currentUser?.accountId
             }, new BN(amount + "00000000000000000000"), BOATLOAD_OF_GAS).then(value => {
                 props.nearContext?.updateNearBalance();
                 props.nearContext?.updateXdhoBalance();
                 props.toastContext?.showSuccess("Вы Успешно купили токены");
             }).catch(reason => {
                 console.log(reason);
                 props.toastContext?.showError(JSON.stringify(JSON.stringify(reason.kind || reason)));
             }).finally(() => {
                 props.setDialogOpen(false);
             })
         }
     }

    const basicDialogFooter = <Button type="button" label={`Купить ${amount * priceStandardTokens} $XDHO`} icon="pi pi-check" className="p-button-secondary"
                                      onClick={() => {
                                          setLoading(true);
                                          onSubmit(amount);
                                      }}
    />;

    return <Dialog header="Покупка $XDHO" visible={props.dialogOpen}  modal
                   footer={basicDialogFooter} onHide={() => props.setDialogOpen(false)}>
        <div className="p-fluid">
            <h5>{`Курс: ${priceStandardTokens} $XDHO за 1 $NEAR`}</h5>
            <div className="field">
                <label htmlFor="supply">На какое число NEAR брать токенов</label>
                <InputNumber id="supply" value={amount} min={1} max={props.nearContext?.currentUser?.balanceNear}
                             onValueChange={(e) => setAmount(e.target.value || 0)}
                             onChange={(e) => {setAmount(e.value || 0)}}
                             className=""/>
            </div>
        </div>
    </Dialog>
}));

const PageHeader = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext,
    toastContext: allStores.toast as ToastContext,
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext }) => {

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
                <NavLink className={""} to={"/"}>Торги</NavLink>
                <NavLink className={""} to={"/registrator"}>Голосование</NavLink>
                <NavLink className={""} to={"/emitent"}>Эмитенту</NavLink>
                <NavLink className={""} to={"/launchpad"}>Лаунчпад</NavLink>
            </div>
            <div className={"user-info"}>
                {currentUser && <>

                    <span className={"xdho-symbol white-section"}>{currentUser.balanceXdho}</span>
                    <div className={"info white-section"}>
                        <span className={"near-symbol"}>{currentUser.balanceNear}</span>
                        <span className={"wallet"} onClick={toggleProfile}>{currentUser.accountId}</span>
                        <OverlayPanel ref={profilePopup} appendTo={document.body} showCloseIcon
                                      className={"profile-popup"}>
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

var App = inject((allStores: Store) => ({
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { nearContext?: NearContext }) => {
    return <main>
        <div className="App layout-theme-light">
            <PageHeader/>
            <div className={"layout-main-container"}>
                <div className="layout-main">
                    {
                        props.nearContext && <Routes>
                            <Route path={"/"} element={<InvestorPage state={new InvestorPageState(props.nearContext)}/>}/>
                            <Route path={"/registrator"}
                                   element={<RegistartorPage state={new RegistartorPageState(props.nearContext)}/>}/>
                            <Route path={"/emitent"}
                                   element={<EmitentPage state={new EmitentPageState(props.nearContext)}/>}/>
                            <Route path={"/launchpad"}
                                   element={<LaunchpadPage state={new LaunchpadPageState(props.nearContext)}/>}/>
                        </Routes>
                    }
                    {
                        !props.nearContext && <div>
                            <span className={"pi pi-spin pi-spinner"}></span>
                            <h2>Подключение к смарт контракту</h2>
                        </div>
                    }
                </div>
            </div>
            <PageFooter/>
        </div>
    </main>
}))

export default App;
