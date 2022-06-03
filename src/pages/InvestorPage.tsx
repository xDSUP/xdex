import React from "react";
import {Tab, TabList, TabPanel, Tabs} from "react-tabs";
import Select from "react-select/base";

class TokenInput extends React.Component<{ }>{
    render() {
        return <>
            <div>
                <div>
                    <span>Bitcoin</span>
                    <div>
                        <span>Balance: 29012</span>
                        <button>max</button>
                    </div>
                </div>
                <div>
                    <div>
                        <Select onMenuClose={()=>{}} onChange={} onMenuOpen={()=>{}} inputValue={} value={} onInputChange={}/>
                        <input/>
                    </div>
                </div>
            </div>
        </>
    }
}

export class InvestorPage extends React.Component<{ }>{
    render() {
        return <>
            <div className={"page-container"}>
                <div className={"token-section"}>
                    <span>Текущая цена</span>
                    <span className={"token"}>55419</span>
                </div>
                <div className={"trader-section"}>
                    <div className={"swapper"}>
                        <Tabs>
                            <TabList>
                                <Tab>Рыночный оддер</Tab>
                                <Tab>Лимитный ордер</Tab>
                            </TabList>
                            <TabPanel>
                                Привет
                            </TabPanel>
                            <TabPanel>
                                Нигга
                            </TabPanel>
                        </Tabs>
                    </div>
                    <div className={"trader-view"}>

                    </div>
                </div>
                <div className={"order-section"}>

                </div>
            </div>
        </>;
    }
}