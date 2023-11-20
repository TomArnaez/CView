import { ScrollArea, Flex, Card, Image } from "@mantine/core";
import { FaFile } from "react-icons/fa";
import classes from "../css/master.module.css";
import { commands } from "../bindings";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/primitives";
import { convert14BArrayToRGBA } from "../utils";
import { useContextMenu } from "mantine-contextmenu";
import { useImageStore } from "../stores/ImageStore";

export const ImageList = (): JSX.Element => {
  const [thumbnails, setThumbnails] = useState<HTMLCanvasElement[]>([]);
  const { imageStacks, currentStackIdx, setStack } = useImageStore((state) => ({
    imageStacks: state.imageStacks,
    currentStackIdx: state.currentStackIndex,
    setStack: state.setStack,
  }));

  const { showContextMenu } = useContextMenu();

  useEffect(() => {
    const loadThumbnails = async () => {
      const loadedThumbnails = await Promise.all(
        imageStacks.map((_, index) => getThumbnail(index))
      );
      setThumbnails(loadedThumbnails);
    };
    loadThumbnails();
  }, [imageStacks]);

  const getThumbnail = async (stackIdx: number): Promise<HTMLCanvasElement> => {
    const data: Uint16Array = new Uint16Array(
      await invoke("get_image_binary", {
        imageIdx: 0,
        stackIdx,
        resize: true,
      })
    );

    const canvas = document.createElement("canvas");
    canvas.width = 100;
    canvas.height = 100;

    const rgba_data = convert14BArrayToRGBA(data, 100, 100);
    const ctx = canvas.getContext("2d");
    if (ctx) {
      const imageData = ctx.createImageData(canvas.width, canvas.height);
      imageData.data.set(rgba_data);
      ctx.putImageData(imageData, 0, 0);
    }
    return canvas;
  };

  return (
    <ScrollArea type="auto">
      {imageStacks.map((stack, stackIdx: number) => (
        <Card
          onContextMenu={showContextMenu([
            {
              key: "Save",
              title: "Save stack as TIFF",
              onClick: () => commands.saveStack(stackIdx),
            },
          ])}
          radius="md"
          padding="lg"
          shadow="sm"
          className={
            stackIdx === currentStackIdx ? classes.selectedCard : classes.card
          }
          onClick={() => setStack(stackIdx)}
        >
          {thumbnails[stackIdx] && (
            <Image
              src={thumbnails[stackIdx].toDataURL()}
              alt={`Thumbnail for stack ${stackIdx}`}
            />
          )}
          {stack.capture != null ? <p>{stack.capture.type}</p> : null}
          <Flex gap="md" justify="center" align="center" direction="row">
            <FaFile />
            <p>{stack.image_handlers.length}</p>
            <p>{stack.timestamp}</p>
          </Flex>
        </Card>
      ))}
    </ScrollArea>
  );
};