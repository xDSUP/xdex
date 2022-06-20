import {inject, observer} from "mobx-react";
import {Store, ToastContext} from "../index";
import {BOATLOAD_OF_GAS, NearContext, TokenRequest} from "../contract/contract";
import React, {useEffect, useState} from "react";
import {Request} from "./Request"
import {Button} from "primereact/button";
import {ProgressBar} from "primereact/progressbar";


export class VotingState {
    title: string = "Название";

}

export const Voting = inject((allStores: Store) => ({
    toastContext: allStores.toast as ToastContext,
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, request: TokenRequest }) => {

    let [isVoted, setVoted] = useState(false);

    const vote = (result: boolean) => {
        props.nearContext?.contract.vote({request_id: props.request.id, vote: result}, BOATLOAD_OF_GAS).then(() =>
            setVoted(true)
        );
    };

    useEffect(() => {
        props.nearContext?.contract.is_voting({voter_id: props.nearContext?.currentUser?.accountId || "", request_id: props.request.id})
            .then(value => setVoted(value));
    }, [])

    return <div>
            <Request request={props.request} />
            <div>
                {
                props.request.votes && <>
                        <span>Проголосовало: </span>
                        <ProgressBar value={props.request.votes.length / 5 * 100} />
                        {
                            !isVoted && <>
                                <Button label="Голос за" onClick={() => {vote(true)}} className="p-button-success p-button-text mr-2 mb-2" />
                                <Button label="Голос против" onClick={() => {vote(false)}} className="p-button-danger p-button-text mr-2 mb-2" />
                            </>
                        }
                        {
                            isVoted && <>
                                <Button label="Вы проголосовали" className="p-button-text mr-2 mb-2" disabled/>
                            </>
                        }

                    </>
            }
            </div>
    </div>
}));
