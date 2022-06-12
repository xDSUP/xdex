import {inject, observer} from "mobx-react";
import {Store, ToastContext} from "../index";
import {NearContext} from "../contract/contract";
import React from "react";


export class VotingState {
    title: string = "Название";

}

export const Voting = inject((allStores: Store) => ({
    toastContext: allStores.toast as ToastContext,
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, state: VotingState }) => {

    return <>
        <div>
            hello2
        </div>
    </>
}));
