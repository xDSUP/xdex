import React, {useMemo} from "react";
import {inject, observer} from "mobx-react";
import {Balance, NearContext, Order, OrderIndex, STANDARD_TOKEN, Store} from "../contract/contract";
import {Button} from "primereact/button";
import {Dropdown} from "primereact/dropdown";
import {action, makeObservable, observable, runInAction} from "mobx";
import {InputNumber} from "primereact/inputnumber";
import {InputText} from "primereact/inputtext";
import {SelectButton} from "primereact/selectbutton";
import {DataTable} from "primereact/datatable";
import {Column} from "primereact/column";

interface Token {
    name: string,
    code: string,
    icon: string
}

enum SwapMode {
    MARKET,
    LIMIT
}

export class InvestorPageState {
    selectModeValues = [
        {name: 'Маркет', code: SwapMode.MARKET},
        {name: 'Лимит', code: SwapMode.LIMIT},
    ];
    @observable
    selectedMode = this.selectModeValues[0].code;
    @observable
    selectedToken = "";
    @observable
    tokenQuantity = 0;
    @observable
    tokenPrice = 0;

    @observable
    orders: OrderIndex[] = [];
    @observable
    spread: Balance[] = [0, 0];
    @observable
    myOrders: Order[] = [];
    updateSelectedMode = action((mode: SwapMode) => {
        this.selectedMode = mode;
        console.log("Новый мод " + mode);
    });
    updateSelectedToken = action((token: string) => {
        this.selectedToken = token;
    });
    updateTokenQuantity = action((quantity: number) => {
        this.tokenQuantity = quantity;
    });
    updateTokenPrice = action((limit: number) => {
        this.tokenPrice = limit;
    });
    updateOrders = action((orders: OrderIndex[]) => {
        this.orders = orders;
    });
    updateMyOrders = action((orders: Order[]) => {
        this.myOrders = orders;
    });
    updateSpread = action((spread: Balance[]) => {
        this.spread = spread;
    });

    constructor() {
        makeObservable(this);
    }

    @observable _isLoadingMyOrders: boolean = false;

    set isLoadingMyOrders(value: boolean) {
        this._isLoadingMyOrders = value;
    }
}

export const InvestorPage = inject((allStores: Store) => ({
        nearContext: allStores.nearContext as NearContext
    }))(observer((props: { nearContext?: NearContext, state: InvestorPageState }) => {
            let updateOrders = () => {
                runInAction(() => {
                    let asks = props.nearContext?.contract.get_ask_orders({
                        token_id: props.state.selectedToken
                    });
                    let bids = props.nearContext?.contract.get_bid_orders({
                        token_id: props.state.selectedToken
                    });
                    Promise.all([asks, bids]).then(value => {
                        console.log(value);
                    });
                })
            };

            let updateMyOrders = () => {
                props.state.isLoadingMyOrders = true;
                runInAction(() => {
                        let context = props.nearContext;
                        if (context?.currentUser) {
                            let orders: Order[] = [];
                            let asks = context.contract.get_orders({
                                account_id: context.currentUser.accountId,
                                token_id: props.state.selectedToken,
                                side: "Ask"
                            });
                            let bids = context.contract.get_orders({
                                account_id: context.currentUser.accountId,
                                token_id: props.state.selectedToken,
                                side: "Bid"
                            });
                            Promise.all([asks, bids]).then(([asks, bids]) => {
                                orders.push(...(asks.map(value => {return {...value, type: "Limit"}})));
                                orders.push(...(bids.map(value => {return {...value, type: "Limit"}})));

                                props.state.updateMyOrders(orders);
                            }).finally(() => props.state.isLoadingMyOrders = false);

                        }
                    }
                )
            };

            let updateSpread = () => {
                runInAction(() => {
                    props.nearContext?.contract.get_current_spread({
                        token_id: props.state.selectedToken
                    }).then(value => {
                        console.log(value);
                        props.state.updateSpread(value);
                    });
                })
            };

            useMemo(() => {
                console.log("Обновляю");
                updateMyOrders();
                updateOrders();
                updateSpread();
            }, [props.state.selectedToken]);

            return <>
                <div className={"page-container grid"}>
                    <div className={"col-12"}>
                        <div>
                            <Dropdown value={props.state.selectedToken}
                                      onChange={(e) => props.state.updateSelectedToken(e.value)}
                                      options={props.nearContext?.tokens.filter(value => value.token_id !== STANDARD_TOKEN)}
                                      optionLabel="token_id" optionValue={"token_id"} placeholder="Выберете"/>
                            <span>Текущая цена</span>
                            <span className={"token"}>55419</span>
                        </div>
                        <div>
                            <span>Обьём</span>
                            <span className={"token"}>+1.69</span>
                        </div>
                    </div>
                    <div className={"col-12 md:col-4"}>
                        <div className={"card"}>
                            <div className="field">
                                <h5>Режим</h5>
                                <SelectButton value={props.state.selectedMode} defaultValue={SwapMode.MARKET}
                                              onChange={(e) => props.state.updateSelectedMode(e.value)}
                                              unselectable={false} optionValue={"code"}
                                              options={props.state.selectModeValues} optionLabel="name"/>
                            </div>
                            <div className="field">
                                <small>Доступно: {props.nearContext?.currentUser?.balanceXdho || 0}</small>
                                <div className="p-inputgroup">
                                    <span className="p-inputgroup-addon">Цена</span>
                                    {props.state.selectedMode === SwapMode.MARKET &&
                                        <InputText id="tokenLimit" value={"Market"} onInput={event => {
                                        }} disabled={true} className={"input-end"}/>
                                    }
                                    {props.state.selectedMode === SwapMode.LIMIT &&
                                        <InputNumber id="tokenLimit" value={props.state.tokenPrice} className={"input-end"}
                                                     onValueChange={(e) => props.state.updateTokenPrice(e.value || 0)}/>
                                    }
                                    <span className="p-inputgroup-addon">{STANDARD_TOKEN}</span>
                                </div>
                            </div>
                            <div className="field">
                                <small>Доступно: {props.nearContext?.currentUser?.balanceOtherTokens.get(props.state.selectedToken) || 0}</small>
                                <div className="p-inputgroup">
                                    <span className="p-inputgroup-addon">Количество</span>
                                    <InputNumber id="tokenQuantity" value={props.state.tokenQuantity}
                                                 className={"input-end"}
                                                 onValueChange={(e) => props.state.updateTokenQuantity(e.value || 0)}/>
                                    <span className="p-inputgroup-addon">{props.state.selectedToken}</span>
                                </div>
                            </div>
                            <div className={"bid-buttons"}>
                                <Button label="Купить" className="p-button-success" style={{marginRight: '.5em'}}/>
                                <Button label="Продать" className="p-button-danger" style={{marginRight: '.5em'}}/>
                            </div>

                        </div>
                        <div className={"card p-fluid"}>
                            Ставочки и нас много = {props.state.spread}
                        </div>
                    </div>
                    <div className={"col-12 md:col-8"}>
                        <div className={"card p-fluid"}>
                            График
                        </div>
                    </div>
                    <div className="col-12">
                        <div className="card">
                            <h5>Открытые ордера</h5>
                            <DataTable value={props.state.myOrders} paginator className="p-datatable-gridlines" showGridlines
                                       rows={10}
                                       dataKey="id" filterDisplay="menu" loading={props.state._isLoadingMyOrders}
                                       responsiveLayout="scroll"
                                       emptyMessage="Нет ордеров">
                                <Column field="order_id" header="Ид" style={{minWidth: '12rem'}}/>
                                <Column field="order_asset" header="Токен" sortable style={{minWidth: '12rem'}}/>
                                <Column field="type" header="Тип ордера" sortable style={{minWidth: '12rem'}}/>
                                <Column field="side" header="Направление" sortable style={{minWidth: '12rem'}}/>
                                <Column field="price" header="Цена" sortable style={{minWidth: '12rem'}}/>
                                <Column field="qty" header="Кол-во" sortable style={{minWidth: '12rem'}}/>
                            </DataTable>
                        </div>
                    </div>
                </div>
            </>;
        }
    ))
;

