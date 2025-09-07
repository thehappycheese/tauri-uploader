import { Accessor, Component, ComponentProps, createSignal, Setter } from "solid-js";
import { Button, SpecialNumberInput } from "./Components";
import { Details, FileField, FileRejection, FileError } from "@kobalte/core/file-field";

type FileValidationState = 
      {type:"PENDING_SERVER_VALIDATION"}
    | {type:"FAILED_LOCAL_VALIDATION", reason:string}
    | {type:"FAILED_SERVER_VALIDATION", reason:string};

type FileItem = {
    file:File,
    file_name:string,
    validation_state:FileValidationState
}

export const Upload:Component<{
    on_accepted_files:(data:File[])=>void,
}> = props =>{
    const [accepted_file_list, _set_accepted_file_list] = createSignal<null|File[]>(null);
    const [rejected_file_list, _set_rejected_file_list] = createSignal<null|FileRejection>(null);
    const set_file_list = (files:File[])=>{
        console.log(files)
        _set_accepted_file_list(files);
        props.on_accepted_files(files);
    }
    const [error_message, set_error_message] = createSignal<null|string>(null);
    const file_rejection_container = <div></div>;
    return (
        <>

            
            <FileField
                multiple
                maxFiles={999}
                minFileSize={1}
                onFileAccept={set_file_list}
                onFileReject={(data) => console.log("data", data)}
                //onFileChange={(data) => console.log("data", data)}
                class="flex flex-col items-center justify-center height-[400px] gap-3"
                // validate={f=>f.name.endsWith(".txt") ? [] : ["FILE_INVALID_TYPE"]}
            >
                <FileField.HiddenInput name="file-upload" />
                <div>
                    <FileField.Label class="input-label">
                        Upload Files
                    </FileField.Label>
                    <FileField.Dropzone 
                        class="flex flex-col gap-4 items-center justify-center border-2 border-dashed border-gray-500 rounded-lg w-full h-full p-10 data-[drag-over]:outline data-[drag-over]:outline-red-400"
                        onDragEnter={e=>e.target.setAttribute("data-drag-over","")}
                        onDragLeave={e=>e.target.removeAttribute("data-drag-over")}
                        onDragEnd={e=>e.target.removeAttribute("data-drag-over")}
                        onDrop={e=>e.target.removeAttribute("data-drag-over")}
                    >
                        Drop Your Files Here or
                        <FileField.Trigger as={Button}>
                            Click to Upload Files
                        </FileField.Trigger>
                    </FileField.Dropzone>
                </div>
                <table class="upload-table">
                    <thead>
                        <tr>
                            <th>Size</th>
                            <th>Name</th>
                            <th>Item</th>
                            <th></th>
                        </tr>
                    </thead>
                    <FileField.ItemList as="tbody">
                        {(file) => (
                            <FileField.Item as="tr">
                                <FileField.ItemSize as="td" />
                                <FileField.ItemName as="td" />
                                <td><input placeholder="0001" value={"0001"} name={"poggers"}/></td>
                                <td><FileField.ItemDeleteTrigger as={Button}>Remove</FileField.ItemDeleteTrigger></td>
                            </FileField.Item>
                        )}
                    </FileField.ItemList>
                </table>
                <FileField.Description class="input-description">Hey Description</FileField.Description>
                <FileField.ErrorMessage class="input-error-message"></FileField.ErrorMessage>
            </FileField>
            {file_rejection_container}
            
        </>
    );
}

function marshal_64() {
    const chunk_size = 64*2**19; // 32 MB
   
    let buffer = new Uint8Array(0);
    let total_bytes_processed = 0;
    return new TransformStream({
        transform(chunk, controller) {
            total_bytes_processed += chunk.byteLength;
           
            const new_buffer = new Uint8Array(buffer.length + chunk.length);
            new_buffer.set(buffer);
            new_buffer.set(chunk, buffer.length);
            buffer = new_buffer;
            while (buffer.length >= chunk_size) {
                const output_chunk = buffer.slice(0, chunk_size);
                controller.enqueue(output_chunk);
                buffer = buffer.slice(chunk_size);
            }
        },
        flush(controller) {
            if (buffer.length > 0) {
                controller.enqueue(buffer);
            }
            // Remove the lengthChunk code entirely
            console.log(`Total bytes processed: ${total_bytes_processed}, final chunk original size: ${buffer.length}`);
        }
    });
}

