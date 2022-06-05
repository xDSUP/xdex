import React from 'react';
import ReactDOM from 'react-dom/client';
import './css/main.css';
import App from './App';
import {Provider} from "mobx-react";
import {initContract, Store} from "./contract/contract";
import {BrowserRouter} from "react-router-dom";


initContract()
    .then((nearContext) => {
        const stores: Store = {
            nearContext: nearContext
        }
        const root = ReactDOM.createRoot(
            document.getElementById('root') as HTMLElement
        );

        root.render(
            <Provider {...stores}>
                <React.StrictMode>
                    <BrowserRouter>
                        <App />
                    </BrowserRouter>
                </React.StrictMode>
            </Provider>
        );
    });

(window as any).global = window;
// @ts-ignore
window.Buffer = window.Buffer || require('buffer').Buffer;


