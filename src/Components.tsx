

import { ComponentProps, createSignal } from "solid-js";

import { Button as KButton } from "@kobalte/core/button";
import { TextField as KTextField } from "@kobalte/core/text-field";
//import "./TextField.css";



export const Button = (props: ComponentProps<typeof KButton>) => {
	return <KButton class="button" {...props} />;
};

export const NavButton = (props: ComponentProps<typeof KButton>) => {
	return <KButton class="nav-button" {...props} />;
};

export const TextField = (props:ComponentProps<typeof KTextField> & {label:string, error_message:string}) => {
	return <KTextField class="flex flex-col gap-2" {...props}>
        <KTextField.Label>{props.label}</KTextField.Label>
        <KTextField.Input class="inline-flex rounded-md p-1 outline-none bg-white text-gray-900 border-gray-200 border placeholder:text-gray-500 focus-visible:outline-2 focus-visible:text-blue-600 outline-offset-2 [data-invalid]:text-red-500 [data-invalid]:border-red-500 hover:border-gray-800"/>
        <KTextField.ErrorMessage class="input-error-message">{props.error_message}</KTextField.ErrorMessage>
    </KTextField>;
};


export const SpecialNumberInput = (props:ComponentProps<typeof KTextField>) => {
    let input_ref: HTMLInputElement;
    let [untouched, set_untouched] = createSignal(true);
    let set_value = (new_value:string)=>{
        set_untouched(false)
        
        const currentPos = input_ref!?.selectionStart || 0;
        let newPos = currentPos;

        if (/^\d{3} \d{2}$/.test(new_value)){
            if (currentPos > 4) {
                newPos = currentPos + 1;
            }
            new_value = new_value.split(" ").join("  ")
        }
        if (/^\d{3}  \d{3}$/.test(new_value)){
            if (currentPos > 4) {
                newPos = currentPos - 1;
            }
            new_value = new_value.split("  ").join(" ")
        }
        props.onChange!(new_value);
        queueMicrotask(() => {
            if (input_ref! && input_ref.setSelectionRange) {
                // Ensure cursor position is within bounds
                const maxPos = new_value.length;
                const clampedPos = Math.min(Math.max(0, newPos), maxPos);
                input_ref.setSelectionRange(clampedPos, clampedPos);
            }
        });
    }
	return <TextField 
        validationState={(untouched() || (props.value && /(^\d{3}  \d{2}$)|(^\d{3} \d{3}$)/.test(props.value)) ? "valid" :"invalid")}
        {...props}
        onChange={set_value}
        label="Special Number"
        error_message="Must be of the format DDD DDD etc"
        required
    />;
};
