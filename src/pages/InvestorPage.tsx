import React, {useState} from "react";
import {inject, observer} from "mobx-react";
import {Balance, BOATLOAD_OF_GAS, isOk, NearContext, Order, Side, STANDARD_TOKEN} from "../contract/contract";
import {Button} from "primereact/button";
import {Dropdown} from "primereact/dropdown";
import {action, makeObservable, observable, runInAction} from "mobx";
import {InputNumber} from "primereact/inputnumber";
import {InputText} from "primereact/inputtext";
import {SelectButton} from "primereact/selectbutton";
import {DataTable} from "primereact/datatable";
import {Column} from "primereact/column";
import {Store, ToastContext} from "../index";

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
    selectedToken = "TEST";
    @observable
    tokenQuantity = 0;
    @observable
    tokenPrice = 0;

    @observable
    orders: FullOrder[] = [];
    @observable
    spread: Balance[] = [0, 0];
    @observable
    myOrders: Order[] = [];
    updateSelectedMode = action((mode: SwapMode) => {
        this.selectedMode = mode;
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
    @observable loadingMyOrders: boolean = false;

    nearContext: NearContext;

    constructor(nearContext: NearContext) {
        makeObservable(this);
        this.nearContext = nearContext;
        this.updateMyOrders();
        this.updateOrders();
        this.updateSpread();
    }

    @action.bound
    setLoadingMyOrders(value: boolean) {
        this.loadingMyOrders = value;
    }

    @action.bound
    updateOrders = () => {
        let asks = this.nearContext?.contract.get_ask_orders({
            token_id: this.selectedToken
        }).then(orders => {
            let mergedAsks = mergeIdenticalPrices(orders);
            return [...mergedAsks.values()].sort((a, b) => b.price - a.price);
        });
        let bids = this.nearContext?.contract.get_bid_orders({
            token_id: this.selectedToken
        }).then(orders => {
            let mergedAsks = mergeIdenticalPrices(orders);
            return [...mergedAsks.values()].sort((a, b) => b.price - a.price);
        });
        Promise.all([asks, bids]).then(value => {
            let asks = value[0] || [];
            let bids = value[1] || [];

            let orders: FullOrder[] = [...asks, ...bids];

            runInAction(() => {
                this.orders = orders;
            })
        });
    };

    @action.bound
    updateMyOrders = () => {
        this.setLoadingMyOrders(true);
        runInAction(() => {
                let context = this.nearContext;
                if (context?.currentUser) {
                    let orders: Order[] = [];
                    let asks = context.contract.get_orders({
                        account_id: context.currentUser.accountId,
                        token_id: this.selectedToken,
                        side: "Ask"
                    });
                    let bids = context.contract.get_orders({
                        account_id: context.currentUser.accountId,
                        token_id: this.selectedToken,
                        side: "Bid"
                    });
                    Promise.all([asks, bids]).then(([asks, bids]) => {
                        orders.push(...(asks.map(value => {
                            return {...value, type: "Limit"}
                        })));
                        orders.push(...(bids.map(value => {
                            return {...value, type: "Limit"}
                        })));
                        runInAction(() => {
                            this.myOrders = orders;
                        })
                    }).finally(() => runInAction(() => this.setLoadingMyOrders(false)));
                }
            }
        )
    };

    @action.bound
    updateSpread = () => {
        runInAction(() => {
            this.nearContext?.contract.get_current_spread({
                token_id: this.selectedToken
            }).then(value => {
                runInAction(() => this.spread = value);
            });
        })
    };
}

interface FullOrder {
    price: number;
    quantity: number;
    order_side: Side;
}

function mergeIdenticalPrices(orders?: Order[]): Map<number, FullOrder> {
    let newOrders = new Map<number, FullOrder>();
    if (orders) {
        for (const order of orders) {
            if (newOrders.has(order.price)) {
                let oldQty = newOrders.get(order.price) || {
                    price: order.price,
                    quantity: order.qty,
                    order_side: order.side
                };
                oldQty.quantity += order.qty;
                newOrders.set(order.price, oldQty);
            } else {
                newOrders.set(order.price, {price: order.price, quantity: order.qty, order_side: order.side});
            }
        }
    }
    return newOrders;
}


export const InvestorPage = inject((allStores: Store) => ({
        toastContext: allStores.toast as ToastContext,
        nearContext: allStores.nearContext as NearContext
    }))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, state: InvestorPageState }) => {
            let [orderProcessing, setOrderProcessing] = useState(false);

            let newBidLimitOrder = () => {
                let state = props.state;
                props.nearContext?.contract.new_limit_order({
                    token_id: state.selectedToken,
                    side: "Bid",
                    quantity: state.tokenQuantity,
                    price: state.tokenPrice
                }, BOATLOAD_OF_GAS).then(value => {
                    props.state.updateMyOrders();
                    props.nearContext?.updateXdhoBalance();
                    props.nearContext?.updateAllBalance();

                    props.state.updateOrders();
                    props.state.updateSpread();

                    props.toastContext?.showSuccess("Лимитный ордер на покупку принят");
                    for (const response of value) {
                        if (!isOk(response))
                            props.toastContext?.showError(`Лимитный ордер ошибка: ${JSON.stringify(response)}`);
                    }
                }).catch(reason => {
                    props.toastContext?.showError(JSON.stringify(reason.kind));
                }).finally(() => setOrderProcessing(false));
            }

            let newBidMarketOrder = () => {
                let state = props.state;
                props.nearContext?.contract.new_market_order({
                    token_id: state.selectedToken,
                    side: "Bid",
                    quantity: state.tokenQuantity,
                }, BOATLOAD_OF_GAS).then(value => {
                    props.nearContext?.updateXdhoBalance();
                    props.nearContext?.updateAllBalance();
                    props.toastContext?.showSuccess("Рыночный ордер на покупку принят");
                    for (const response of value) {
                        if (!isOk(response))
                            props.toastContext?.showError(`Рыночный ордер ошибка: ${JSON.stringify(response)}`);
                    }
                    props.state.updateOrders();
                    props.state.updateSpread();
                }).catch(reason => {
                    props.toastContext?.showError(JSON.stringify(reason.kind));
                }).finally(() => setOrderProcessing(false))
            }

            let newAskLimitOrder = () => {
                let state = props.state;
                props.nearContext?.contract.new_limit_order({
                    token_id: state.selectedToken,
                    side: "Ask",
                    quantity: state.tokenQuantity,
                    price: state.tokenPrice
                }, BOATLOAD_OF_GAS).then(value => {
                    props.state.updateMyOrders();
                    props.nearContext?.updateXdhoBalance();
                    props.nearContext?.updateAllBalance();
                    props.toastContext?.showSuccess("Лимитный ордер на продажу принят");
                    for (const response of value) {
                        if (!isOk(response))
                            props.toastContext?.showError(`Лимитный ордер ошибка: ${JSON.stringify(response)}`);
                    }
                    props.state.updateOrders();
                    props.state.updateSpread();
                }).catch(reason => {
                    props.toastContext?.showError(JSON.stringify(reason.kind));
                }).finally(() => setOrderProcessing(false))
            }

            let newAskMarketOrder = () => {
                let state = props.state;
                props.nearContext?.contract.new_market_order({
                    token_id: state.selectedToken,
                    side: "Ask",
                    quantity: state.tokenQuantity,
                }, BOATLOAD_OF_GAS).then(value => {
                    props.nearContext?.updateXdhoBalance();
                    props.nearContext?.updateAllBalance();
                    props.toastContext?.showSuccess("Рыночный ордер на продажу принят");
                    for (const response of value) {
                        if (!isOk(response))
                            props.toastContext?.showError(`Рыночный ордер ошибка: ${JSON.stringify(response)}`);
                    }
                    props.state.updateOrders();
                    props.state.updateSpread();
                }).catch(reason => {
                    props.toastContext?.showError(JSON.stringify(reason.kind));
                }).finally(() => setOrderProcessing(false))
            }

            let deleteLimit = (id: number, side: Side) => {
                let state = props.state;
                props.nearContext?.contract.cancel_limit_order({
                    token_id: state.selectedToken,
                    side: side,
                    id: id,
                }, BOATLOAD_OF_GAS).then(value => {
                    props.state.updateMyOrders();
                    props.nearContext?.updateXdhoBalance();
                    props.nearContext?.updateAllBalance();
                    props.state.updateOrders();
                    props.state.updateSpread();
                    for (const response of value) {
                        if(isOk(response))
                            props.toastContext?.showSuccess(`Лимитный ордер №${id} успешно отменен`);
                        else
                            props.toastContext?.showError(`Лимитный ордер №${id} не отменен с ошибкой: ${JSON.stringify(response)}`);
                    }

                }).catch(reason => {
                    props.toastContext?.showError(JSON.stringify(JSON.stringify(reason.kind)));
                }).finally(() => setOrderProcessing(false));
            }

            return <>
                <div className={"page-container grid"}>
                    <div className={"col-12 sm:col-4"}>
                        <div className={"card flex justify-content-between"}>
                            <Dropdown value={props.state.selectedToken}
                                      onChange={(e) => props.state.updateSelectedToken(e.value)}
                                      options={props.nearContext?.tokens.filter(value => value.token_id !== STANDARD_TOKEN)}
                                      optionLabel="token_id" optionValue={"token_id"} placeholder="Выберете"
                                      className={"text-4xl"}
                            />
                            <div className={"flex flex-column"}>
                                <span className={""}>Текущая цена</span>
                                <span className={""}>{props.state.spread[0]}</span>
                            </div>
                        </div>
                    </div>
                    <div className={"col-12 sm:col-4"}>
                        <div className={"card"}>
                            <span className={"block"}>Обьём за 24 часа</span>
                            <span className={"block"}>+1.69</span>
                        </div>
                    </div>
                    <div className={"col-12 sm:col-4"}>
                        <div className={"card"}>
                            <span className={"block"}>Всего токенов в обороте</span>
                            <span
                                className={"block"}>{props.nearContext?.tokensMap.get(props.state.selectedToken)?.supply || 0}</span>
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
                                <Button label="Купить" className="p-button-success" disabled={orderProcessing}

                                        onClick={() => {
                                            setOrderProcessing(true);
                                            props.state.selectedMode === SwapMode.LIMIT
                                                ? newBidLimitOrder()
                                                : newBidMarketOrder()
                                        }}
                                        style={{marginRight: '.5em'}}/>
                                <Button label="Продать" className="p-button-danger" disabled={orderProcessing}
                                        onClick={() => {
                                            setOrderProcessing(true);
                                            props.state.selectedMode === SwapMode.LIMIT
                                                ? newAskLimitOrder()
                                                : newAskMarketOrder()
                                        }}
                                        style={{marginRight: '.5em'}}/>
                            </div>
                        </div>
                    </div>


                    <div className="col-12 md:col-8">
                        <div className="card">
                            <h5>Мои открытые ордера</h5>
                            <DataTable value={props.state.myOrders} paginator className="p-datatable-gridlines"
                                       rows={10}
                                       dataKey="id" filterDisplay="menu" loading={props.state.loadingMyOrders}
                                       responsiveLayout="scroll" tableClassName={"bids-table"}
                                       emptyMessage="Нет ордеров">
                                <Column field="order_id" header="Ид" style={{minWidth: '5rem'}}/>
                                <Column field="order_asset" header="Токен" sortable style={{minWidth: '8rem'}}/>
                                <Column field="type" header="Тип ордера" sortable style={{minWidth: '7rem'}}/>
                                <Column field="side" header="Сделка" sortable style={{minWidth: '7rem'}}/>
                                <Column field="price" header="Цена" sortable style={{minWidth: '12rem'}}/>
                                <Column field="qty" header="Кол-во" sortable style={{minWidth: '12rem'}}/>
                                <Column header="" style={{minWidth: '1rem'}} body={(rowData) => {
                                    return <Button className={"p-0"} icon={"pi pi-times"} onClick={() => {deleteLimit(rowData.order_id, rowData.side)}}></Button>
                                }}/>
                            </DataTable>
                        </div>
                    </div>
                </div>
                <div className={"col-12 md:col-4"}>
                    <div className={"card p-fluid"}>
                        <span>{`Спред: [${props.state.spread[0]}, ${props.state.spread[1]}]`}</span>
                        <DataTable value={props.state.orders} className="p-datatable-gridlines"
                                   rows={10} tableClassName={"orders-table"} showGridlines={false}
                                   rowClassName={data => data.order_side == "Ask" ? "ask-order" : "bid-order"}
                                   emptyMessage="Нет ордеров">
                            <Column field="price" header="Цена XDHO" style={{minWidth: '6rem'}}/>
                            <Column field="quantity" header={`Объём ${props.state.selectedToken}`}
                                    style={{minWidth: '7rem', textAlign: 'end'}}/>
                            <Column field="total" header="Обьём XDHO" style={{minWidth: '7rem', textAlign: 'end'}}/>
                        </DataTable>
                    </div>
                </div>

            </>;
        }
    ))
;
/**
 * <div className={"col-12 md:col-8"}>
 *                         <div className={"card p-fluid"}>
 *                             График
 *                         </div>
 *                     </div>
 */
