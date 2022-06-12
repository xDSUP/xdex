import React from "react";
import {inject, observer} from "mobx-react";
import {Store, ToastContext} from "../index";
import {NearContext} from "../contract/contract";



export class RegistartorPageState {

}

export const RegistartorPage = inject((allStores: Store) => ({
    toastContext: allStores.toast as ToastContext,
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, state: RegistartorPageState }) => {
    return <>
        <div className="grid">
            <div className="col-12 sm:col-6 md:col-3">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Всего заявок</span>
                            <div className="text-900 font-medium text-xl">10</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-blue-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-shopping-cart text-blue-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">5 новых </span>
                    <span className="text-500">за 24 часа</span>
                </div>
            </div>
            <div className="col-12 sm:col-6 md:col-3">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Токены в стейкинге</span>
                            <div className="text-900 font-medium text-xl">2.100.000</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-orange-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-map-marker text-orange-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">+%52 </span>
                    <span className="text-500">за последнюю неделю</span>
                </div>
            </div>
            <div className="col-12 sm:col-6 md:col-3">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Регистраторов</span>
                            <div className="text-900 font-medium text-xl">10</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-cyan-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-inbox text-cyan-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">+2 </span>
                    <span className="text-500">новых</span>
                </div>
            </div>
            <div className="col-12 sm:col-6 md:col-3">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Одобрено токенов</span>
                            <div className="text-900 font-medium text-xl">{props.nearContext?.tokens.length || 0}</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-purple-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-comment text-purple-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">+4 </span>
                    <span className="text-500">новых за месяц</span>
                </div>
            </div>
        </div>
        <h1>Активные голосования</h1>
        <div className="grid">
            <div className="col-12 sm:col-6 md:col-4">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Всего заявок</span>
                            <div className="text-900 font-medium text-xl">10</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-blue-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-shopping-cart text-blue-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">5 новых </span>
                    <span className="text-500">за 24 часа</span>
                </div>
            </div>
            <div className="col-12 sm:col-6 md:col-4">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Токены в стейкинге</span>
                            <div className="text-900 font-medium text-xl">2.100.000</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-orange-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-map-marker text-orange-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">+%52 </span>
                    <span className="text-500">за последнюю неделю</span>
                </div>
            </div>
            <div className="col-12 sm:col-6 md:col-4">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Регистраторов</span>
                            <div className="text-900 font-medium text-xl">10</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-cyan-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-inbox text-cyan-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">+2 </span>
                    <span className="text-500">новых</span>
                </div>
            </div>
            <div className="col-12 sm:col-6 md:col-4">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Одобрено токенов</span>
                            <div className="text-900 font-medium text-xl">{props.nearContext?.tokens.length || 0}</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-purple-100 border-round"
                             style={{width: '2.5rem', height: '2.5rem'}}>
                            <i className="pi pi-comment text-purple-500 text-xl"/>
                        </div>
                    </div>
                    <span className="text-green-500 font-medium">+4 </span>
                    <span className="text-500">новых за месяц</span>
                </div>
            </div>
        </div>
    </>
}));