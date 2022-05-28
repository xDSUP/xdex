/// <reference types="react-scripts" />
export {};

declare global {
    namespace NodeJS {
        interface ProcessEnv {
            NEAR_NO_LOGS: boolean;
            REACT_APP_CONTRACT_NAME: string;
            REACT_APP_MASTER_ACCOUNT: string;
        }
    }
}
