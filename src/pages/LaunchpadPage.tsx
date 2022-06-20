import React from "react";
import {inject, observer} from "mobx-react";
import {Store, ToastContext} from "../index";
import {NearContext} from "../contract/contract";
import {create} from "ipfs-http-client";
import {Button} from "primereact/button";

const client = create({url: "https://ipfs.infura.io:5001/api/v0"});

export class LaunchpadPageState {
    nearContext: NearContext;
    constructor(nearContext: NearContext) {
        //makeObservable(this);
        this.nearContext = nearContext;
    }
}

export const LaunchpadPage = inject((allStores: Store) => ({
    toastContext: allStores.toast as ToastContext,
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, state: LaunchpadPageState }) => {
    return <>

        <div className="grid">
            <div className="col-12 sm:col-6">
                <div className="card mb-0">
                    <div className="flex justify-content-between mb-3">
                        <div>
                            <span className="block text-500 font-medium mb-3">Токен [NAME]</span>
                            <div className="text-900 font-medium text-xl">НАЗВАНИЕ И ОПИСАНИЕ</div>
                        </div>
                        <div className="flex align-items-center justify-content-center bg-blue-100 border-round"
                             style={{width: "2.5rem", height: "2.5rem"}}>
                            <i className="pi pi-id-card text-blue-500 text-xl"/>
                        </div>
                        <span>Текущая цена</span>
                        <Button type="button" label="Купить токен" icon="pi pi-file"
                                onClick={() => {

                                }}/>
                    </div>
                </div>
            </div>


        </div>
    </>
}));