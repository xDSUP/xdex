import React, {RefObject} from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import {Provider} from "mobx-react";
import {initContract, NearContext} from "./contract/contract";
import {BrowserRouter} from "react-router-dom";
import {Toast} from "primereact/toast";

import 'primereact/resources/primereact.css';
import 'primeicons/primeicons.css';
import 'primeflex/primeflex.css';
import './css/main.css';

export interface Store {
    nearContext: NearContext,
    toast?: ToastContext
}

const TOAST_TIME_LIFE = 3000;

export class ToastContext {
    toast: RefObject<Toast>;

    constructor(toast: RefObject<Toast>) {
        this.toast = toast;
    }

    showSuccess(detail:string) {
        this.toast.current?.show({
            severity: 'success',
            summary: 'Успех',
            detail: detail,
            life: TOAST_TIME_LIFE
        });
    };

    showInfo(detail:string) {
        this.toast.current?.show({severity: 'info', summary: 'Уведомление', detail: detail, life: TOAST_TIME_LIFE});
    };

    showWarn(detail:string) {
        this.toast.current?.show({severity: 'warn', summary: 'Предупреждение', detail: detail, life: TOAST_TIME_LIFE});
    };

    showError(detail:string) {
        this.toast.current?.show({severity: 'error', summary: 'Произошла ошибка', detail: detail, life: TOAST_TIME_LIFE});
    };
}

initContract()
    .then((nearContext) => {
        let toast = React.createRef<Toast>();

        const stores: Store = {
            nearContext: nearContext,
            toast: new ToastContext(toast),
        }
        const root = ReactDOM.createRoot(
            document.getElementById('root') as HTMLElement
        );

        root.render(
            <Provider {...stores}>
                <React.StrictMode>
                    <BrowserRouter>
                        <Toast ref={toast}/>
                        <App/>
                    </BrowserRouter>
                </React.StrictMode>
            </Provider>
        );
    });

(window as any).global = window;
// @ts-ignore
window.Buffer = window.Buffer || require('buffer').Buffer;


