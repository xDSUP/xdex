import {observer} from "mobx-react";
import React, {useState} from "react";
import axios from "axios";
import MDEditor from "@uiw/react-md-editor";
import rehypeSanitize from "rehype-sanitize";

export interface TokenRequest {
    id: number
    token_id: string;
    title: string;
    description: string;
    price: number;
    supply: string;
    // hash
    info: string;
}

export const Request = observer((props: { request: TokenRequest }) => {
    const [loading, setLoading] = useState(true);
    const [text, setText] = useState("");
    const request = props.request;

    const url = `https://ipfs.infura.io/ipfs/${request.info}`;
    axios.get(url, {responseType: "text"})
        .then(info => {
            setLoading(false);
            setText(info.data);
        });

    return <div className={"col-12 md:col-6"}>
        <div className={"card"}>
            <div className={"flex flex-row justify-content-between"}>
                <h4>{`${request.title} ($${request.token_id}) №${request.id}`}</h4>
                <div className={"flex flex-column"}>
                    <span>{`Цена за единицу: ${request.price} $XDHO`}</span>
                    <span>{`Количество токенов: ${request.supply}`}</span>
                </div>
            </div>

            <div>
                <span>{`Описание: ${request.description}`}</span>
            </div>
            <h5>Текст заявки:</h5>
            {
                loading && <>
                    <i className={"pi pi-spin pi-spinner"}></i>
                    <span>Загружаюсь</span>
                </>
            }
            {
                !loading && <>
                    <MDEditor.Markdown
                        source={text}
                        rehypePlugins={[[rehypeSanitize]]}
                    />
                </>
            }
        </div>

    </div>
});