import { Accessor, createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import {Upload} from "./Upload"
import { SpecialNumberInput } from "./Components";
import logo from "./assets/128x128.png"

type UserInfo = {
    display_name: string,
    mail: string,
    user_principal_name:string,
    id:string
}

type LoginState = {
    type:"LOGGED_OUT"
} | {
    type:"AWAITING_LOGIN"
}| {
    type:"LOGGED_IN",
    user_info:UserInfo
}

function App() {
    const [artifact_name, set_artifact_name] = createSignal("");
    const [sv, set_sv] = createSignal("");
    const [name, setName] = createSignal("");
    const [ls, set_ls] = createSignal<LoginState>({type:"LOGGED_OUT"})

    async function login() {
        set_ls({type:"AWAITING_LOGIN"});
        let user_info:UserInfo = await invoke("login");
        console.log(user_info)
        set_ls({
            type:"LOGGED_IN",
            user_info,
        })
    }
    function narrow<A, B extends A>(value:Accessor<A>, condition:(a:A)=>a is B):B|null{
        let val = value();
        if (condition(val)){
            return val
        }else{
            return null
        }
    }

    return (
        <main class="container">
            <div class="flex justify-center items-center gap-5">
                <img src={logo}/>
                <h1 class="text-3xl p-5">Uploader</h1>
            </div>

            <p class="m-5">{ls().type} <button type="button" onClick={()=>login()}>Login</button></p>
            
            <Show when={narrow(ls, ls=>ls.type==="LOGGED_IN")}>{ l =>
                <p class="m-5">{l().user_info.display_name}</p>
            }</Show>

            <form
                class="flex flex-col gap-4"
                onSubmit={(e) => {
                    e.preventDefault();
                }}
            >
                <div class="flex gap-5 justify-center items-center">
                    <label for="inp1">Artifact Name</label>
                    <input id="inp1" type="text" onChange={set_artifact_name} value={artifact_name()}/>
                </div>
                <SpecialNumberInput
                    value={sv()}
                    onChange={set_sv}
                ></SpecialNumberInput>
                <Upload
                    on_accepted_files={(f)=>{}}
                />
                    
                
            </form>
            <p>{artifact_name()}</p>
        </main>
    );
}

export default App;
