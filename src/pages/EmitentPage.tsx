import React from "react";
import {inject, observer} from "mobx-react";
import {Store, ToastContext} from "../index";
import {NearContext} from "../contract/contract";
import {Dialog} from "primereact/dialog";
import {Button} from "primereact/button";
import {action, makeObservable, observable, runInAction} from "mobx";
import {InputNumber} from "primereact/inputnumber";
import {InputText} from "primereact/inputtext";
import MDEditor from '@uiw/react-md-editor';
import rehypeSanitize from "rehype-sanitize";
import {create} from "ipfs-http-client";
import {Request, TokenRequest} from "../controls/Request";

const client = create({url: "https://ipfs.infura.io:5001/api/v0"});

export class EmitentPageState {
    @observable dialogOpen = false;
    @observable tokenId = "";
    @observable title = "";
    @observable description = "";
    @observable price = 1;
    @observable supply = 0;
    @observable info = "";
    @observable urls: string[] = [];

    constructor() {
        makeObservable(this);
    }

    @action.bound
    setTokenId(value: string) {
        this.tokenId = value;
    }

    @action.bound
    setTitle(value: string) {
        this.title = value;
    }

    @action.bound
    setDescription(value: string) {
        this.description = value;
    }

    @action.bound
    setPrice(value: number) {
        this.price = value;
    }

    @action.bound
    setSupply(value: number) {
        this.supply = value;
    }

    @action.bound
    setInfo(value: string | void | undefined) {
        this.info = value || "";
    }

    @action.bound
    addUrl(value: string) {
        this.urls.push(value);
    }

    @action.bound
    setDialogOpen(value: boolean) {
        this.dialogOpen = value;
    }
}

export const EmitentPage = inject((allStores: Store) => ({
    toastContext: allStores.toast as ToastContext,
    nearContext: allStores.nearContext as NearContext
}))(observer((props: { toastContext?: ToastContext, nearContext?: NearContext, state: EmitentPageState }) => {
    const onSubmit = () => {
        var data = new Blob([props.state.info], {type: 'text/plain'});
        client.add(data).then(
            (result) => {
                const url = `https://ipfs.infura.io/ipfs/${result.path}`;
                console.log(url);
                runInAction(() => {
                    props.state.addUrl(url);
                })
            }
        );
    };

    const basicDialogFooter = <Button type="button" label="Dismiss" icon="pi pi-check" className="p-button-secondary"
                                      onClick={() => {
                                          props.state.setDialogOpen(false);
                                          onSubmit();
                                      }}
    />;

    const getMyTokenRequests = (): TokenRequest[] => {
        return [0,1,2,3].map(v => ({
            id: 123,
            description: "Сосисочка - тестовый токен нового покеления",
            title: "SOSISKA",
            info: "bafybeiexs4lolbgjrih32xpimul6iwjc7q5bm5wywsogfnioi5ulc2dqf4",
            token_id: "SOSI",
            price: 1.312,
            supply: "100000",
        }));

    }

    return <>
        <div><Dialog header="Dialog" visible={props.state.dialogOpen} style={{width: '30vw'}} modal
                     footer={basicDialogFooter} onHide={() => props.state.setDialogOpen(false)}>
            <div className="grid w-600">
                <div className="col-12 md:col-6 p-fluid">
                    <div className="field">
                        <label htmlFor="title">Название токена</label>
                        <InputText type="text" id="title" value={props.state.title}
                                   onChange={(e) => props.state.setTitle(e.target.value)} className=""/>
                    </div>
                    <div className="field">
                        <label htmlFor="token-id">Сокращение токена</label>
                        <InputText type="text" id="token-id" value={props.state.tokenId}
                                   onChange={(e) => props.state.setTokenId(e.target.value)} className=""/>
                    </div>
                </div>
                <div className="col-12 md:col-6 p-fluid">
                    <div className="field">
                        <label htmlFor="desc">Описание токена</label>
                        <InputText type="text" id="desc" value={props.state.description}
                                   onChange={(e) => props.state.setDescription(e.target.value)} className=""/>
                    </div>
                    <div className="field">
                        <label htmlFor="price">Цена за единицу на старте</label>
                        <InputNumber id="price" value={props.state.price} min={1}
                                     onValueChange={(e) => props.state.setPrice(e.target.value || 0)} className=""/>
                    </div>
                </div>

                <div className="col-12 md:col-6 p-fluid">
                    <div className="field">
                        <label htmlFor="supply">Количество токенов</label>
                        <InputNumber id="supply" value={props.state.supply} min={1}
                                     onValueChange={(e) => props.state.setSupply(e.target.value || 0)} className=""/>
                    </div>
                </div>
                <div className="col-12">
                    <div className="container">
                        <MDEditor
                            value={props.state.info}
                            onChange={props.state.setInfo}
                            previewOptions={{
                                rehypePlugins: [[rehypeSanitize]],
                            }}
                        />
                    </div>
                </div>
            </div>
        </Dialog>
            <div className="grid">
                <div className="col-12">
                    <Button type="button" label="Создать заявку" icon="pi pi-file"
                            onClick={() => {
                                props.state.setDialogOpen(true);
                            }}/>
                </div>
            </div>
            <div>
                <h3>Мои заявки:</h3>
                <div className={"grid"}>
                    {
                        getMyTokenRequests().map((request) => <Request request={request}/>)
                    }
                </div>

            </div>

        </div>
    </>
}));