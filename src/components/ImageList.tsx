import { ScrollArea, Flex, Card, Image } from "@mantine/core";
import { FaFile } from "react-icons/fa";
import classes from "../css/master.module.css";
import { commands } from "../bindings";
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/primitives";
import { camelCaseToWords } from "../utils";
import { useContextMenu } from "mantine-contextmenu";
import { useImageStore } from "../stores/ImageStore";

export const ImageList = (): JSX.Element => {
  const thumbnailSize = 300;

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
    const image_data: Uint8Array = new Uint8Array(
      await invoke("get_image_binary_rgba", {
        imageIdx: 0,
        stackIdx,
        resizeSize: thumbnailSize,
      })
    );

    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");
    if (ctx) {
      const imageData = ctx.createImageData(thumbnailSize, thumbnailSize);
      imageData.data.set(image_data);
      ctx.putImageData(imageData, 0, 0);
    }
    return canvas;
  };

  return (
    <ScrollArea type="auto">
      {imageStacks.map((stack, stackIdx: number) => {
        const conditionalStyle = stackIdx === currentStackIdx ? { backgroundColor: 'lightgray' } : {};

        return (
          <Card
            style={conditionalStyle}
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
            className={classes.SelectedCard}
            onClick={() => setStack(stackIdx)}
            key={stackIdx}
          >
            {thumbnails[stackIdx] && (
              <Image
                src={thumbnails[stackIdx].toDataURL()}
                alt={`Thumbnail for stack ${stackIdx}`}
              />
            )}
            {stack.capture != null ? <p>{camelCaseToWords(stack.capture.type)}</p> : null}
            <Flex gap="md" justify="center" align="center" direction="row">
              <FaFile />
              <p>{stack.image_handlers.length}</p>
              <p>{stack.timestamp}</p>
            </Flex>
          </Card>
        );
      })}
    </ScrollArea>
  );
}