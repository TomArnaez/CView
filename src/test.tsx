import React, { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import * as Form from "@radix-ui/react-form";
import { Cross2Icon } from "@radix-ui/react-icons";

const DialogDemo = () => {
  const [number, setNumber] = useState<string>("");
  const [list, setList] = useState<number[]>([]);

  const handleAdd = () => {
    const numberValue = parseFloat(number);
    if (numberValue > 0 && !list.includes(numberValue)) {
      setList([...list, numberValue]);
      setNumber("");
    }
  };

  const handleRemove = (index: number) => {
    setList(list.filter((_, idx) => idx !== index));
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      handleAdd();
    }
  };

  return (
    <Dialog.Root>
      <Dialog.Trigger asChild>
        <button className="text-violet11 shadow-blackA4 hover:bg-mauve3 inline-flex h-[35px] items-center justify-center rounded-[4px] bg-white px-[15px] font-medium leading-none shadow-[0_2px_10px] focus:shadow-[0_0_0_2px] focus:shadow-black focus:outline-none">
          Edit profile
        </button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="bg-blackA6 data-[state=open]:animate-overlayShow fixed inset-0" />
        <Dialog.Content className="data-[state=open]:animate-contentShow fixed top-[50%] left-[50%] max-h-[85vh] w-[90vw] max-w-[450px] translate-x-[-50%] translate-y-[-50%] rounded-[6px] bg-white p-[25px] shadow-[hsl(206_22%_7%_/_35%)_0px_10px_38px_-10px,_hsl(206_22%_7%_/_20%)_0px_10px_20px_-15px] focus:outline-none">
          <Dialog.Title className="text-mauve12 m-0 text-[17px] font-medium">
            Provide Exposure Times
          </Dialog.Title>
          <Form.Root onSubmit={() => console.log("hey")} className="w-[260px]">
            <Form.Field className="grid mb-[10px]" name="Exposure Times (ms)">
              <Form.Control asChild>
                <>
                  <input
                    type="number"
                    value={number}
                    onChange={(e) => setNumber(e.target.value)}
                    className="mt-4 p-2 border rounded w-full"
                    onKeyDown={handleKeyDown}
                  />
                  <div className="flex flex-wrap gap-2 mt-4">
                    {list.map((item, index) => (
                      <span
                        key={index}
                        className="bg-blue-200 cursor-pointer px-3 py-1 rounded-full"
                        onClick={() => handleRemove(index)}
                      >
                        {item}
                      </span>
                    ))}
                  </div>
                </>
              </Form.Control>
            </Form.Field>
            <Form.Submit asChild>
              <button className="box-border w-full text-violet11 shadow-blackA4 hover:bg-mauve3 inline-flex h-[35px] items-center justify-center rounded-[4px] bg-white px-[15px] font-medium leading-none shadow-[0_2px_10px] focus:shadow-[0_0_0_2px] focus:shadow-black focus:outline-none mt-[10px]">
                Post question
              </button>
            </Form.Submit>
          </Form.Root>

          <Dialog.Close asChild>
            <button
              className="text-violet11 hover:bg-violet4 focus:shadow-violet7 absolute top-[10px] right-[10px] inline-flex h-[25px] w-[25px] appearance-none items-center justify-center rounded-full focus:shadow-[0_0_0_2px] focus:outline-none"
              aria-label="Close"
            >
              <Cross2Icon />
            </button>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default DialogDemo;
