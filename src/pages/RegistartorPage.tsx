import React, {useState} from "react";
import {inject, observer} from "mobx-react";
import {Store, ToastContext} from "../index";
import {NearContext, TokenRequest} from "../contract/contract";
import {makeObservable, observable, runInAction} from "mobx";
import {Voting} from "../controls/Voting";
import moment from "moment/moment";

interface PageInfo {
    staked_tokens_count: number,
    stakers_count: number,
    approved_tokens_count: number
}

export class RegistartorPageState {
    @observable
    requests: TokenRequest[] = [];
    @observable
    pageInfo: PageInfo = {
        approved_tokens_count: 0,
        staked_tokens_count: 0,
        stakers_count: 0
    };


    nearContext: NearContext;

    constructor(nearContext: NearContext) {
        makeObservable(this);
        this.nearContext = nearContext;
        this.updatePageInfo();
        this.updateRequests();
    }

    updatePageInfo() {
        let promises = []
        let pageInfo: PageInfo = {
            approved_tokens_count: this.nearContext?.tokens.length || 0,
            staked_tokens_count: 0,
            stakers_count: 0
        }
        promises.push(this.nearContext?.contract.get_count_stakers().then(value => pageInfo.stakers_count = value));
        promises.push(this.nearContext?.contract.get_all_staked().then(value => pageInfo.staked_tokens_count = value));
        // всегда есть один токен платформы, тч его не считаем
        promises.push(this.nearContext?.contract.get_launchpad_tokens().then(value => pageInfo.approved_tokens_count += value.length - 1));

        Promise.all(promises).then(() => runInAction(() => {
            this.pageInfo = pageInfo;
        }));
    }

    updateRequests() {
        this.nearContext?.contract.get_all_requests().then(requests => {
            requests = requests.filter(value => {
                let created_date = new Date(Number(("" + value.created_time).slice(0, 13)));
                return moment().diff(moment(created_date), "days") < 1;
            });
            runInAction(() => {
                this.requests = requests;
            })
            let promises = []
            for (let request of requests) {
                console.log("Запрашиваю голоса для " + request.id);
                promises.push(this.nearContext?.contract.get_all_votes({request_id: request.id}).then(value => request.votes = value));
            }
            Promise.all(promises).then(() => runInAction(() => {
                this.requests = requests;
            }));
        });
    }
}

const RegistratorInfoHeader = observer((props: { toastContext?: ToastContext; nearContext?: NearContext; state: RegistartorPageState }) => {
    return <div className="grid">
        <div className="col-12 sm:col-6 md:col-3">
            <div className="card mb-0">
                <div className="flex justify-content-between mb-3">
                    <div>
                        <span className="block text-500 font-medium mb-3">Всего заявок</span>
                        <div className="text-900 font-medium text-xl">{props.state.requests.length}</div>
                    </div>
                    <div className="flex align-items-center justify-content-center bg-blue-100 border-round"
                         style={{width: "2.5rem", height: "2.5rem"}}>
                        <i className="pi pi-id-card text-blue-500 text-xl"/>
                    </div>
                </div>
            </div>
        </div>
        <div className="col-12 sm:col-6 md:col-3">
            <div className="card mb-0">
                <div className="flex justify-content-between mb-3">
                    <div>
                        <span className="block text-500 font-medium mb-3">В стейкинге</span>
                        <div className="text-900 font-medium text-xl">{props.state.pageInfo.staked_tokens_count}</div>
                    </div>
                    <div className="flex align-items-center justify-content-center bg-orange-100 border-round"
                         style={{width: "2.5rem", height: "2.5rem"}}>
                        <i className="pi pi-money-bill text-orange-500 text-xl"/>
                    </div>
                </div>
            </div>
        </div>
        <div className="col-12 sm:col-6 md:col-3">
            <div className="card mb-0">
                <div className="flex justify-content-between mb-3">
                    <div>
                        <span className="block text-500 font-medium mb-3">Регистраторов</span>
                        <div className="text-900 font-medium text-xl">{props.state.pageInfo.stakers_count}</div>
                    </div>
                    <div className="flex align-items-center justify-content-center bg-cyan-100 border-round"
                         style={{width: "2.5rem", height: "2.5rem"}}>
                        <i className="pi pi-users text-cyan-500 text-xl"/>
                    </div>
                </div>
            </div>
        </div>
        <div className="col-12 sm:col-6 md:col-3">
            <div className="card mb-0">
                <div className="flex justify-content-between mb-3">
                    <div>
                        <span className="block text-500 font-medium mb-3">Одобрено токенов</span>
                        <div
                            className="text-900 font-medium text-xl">{props.state.pageInfo.approved_tokens_count}</div>
                    </div>
                    <div className="flex align-items-center justify-content-center bg-purple-100 border-round"
                         style={{width: "2.5rem", height: "2.5rem"}}>
                        <i className="pi pi-check-circle text-purple-500 text-xl"/>
                    </div>
                </div>
            </div>
        </div>
    </div>;
})

export const RegistartorPage = inject((allStores: Store) => ({
    toastContext: allStores.toast as ToastContext,
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, state: RegistartorPageState }) => {
    let [loading, setLoading] = useState(true);
    return <>
        <RegistratorInfoHeader {...props}/>
        <h1>Активные голосования</h1>
        <div className="grid">
            {
                props.state.requests.map(value => <div key={value.created_time} className={"col-12 md:col-6"}>
                        <Voting request={value}/>
                    </div>
                )
            }
        </div>
    </>
}));