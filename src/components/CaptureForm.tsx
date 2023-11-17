import { Button, Progress, Flex, Center, Text } from "@mantine/core";
import { Carousel } from "@mantine/carousel";
import { emit, listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";

type CaptureFormProps = {
  setFormOpen: React.Dispatch<React.SetStateAction<boolean>>;
};

const CaptureForm = ({ setFormOpen }: CaptureFormProps): JSX.Element => {
  const [captureProgress, setCaptureProgress] = useState<number>(0);
  const [captureMessage, setCaptureMessage] = useState<string>("");

  useEffect(() => {
    const unlistenCaptureProgress = listen(
      "capture-progress-event",
      (event) => {
        console.log("received capture-progress-event");
        setCaptureProgress(event.payload.progress);
        setCaptureMessage(event.payload.message);
      }
    );

    const unlistenCaptureComplete = listen("capture-complete-event", () => {
      console.log("received capture-complete-event");
      setCaptureProgress(100);
      setFormOpen(false);
    });

    return () => {
      unlistenCaptureProgress.then((f) => f());
      unlistenCaptureComplete.then((f) => f());
    };
  }, []);

  const handleCancelCapture = async () => {
    emit("cancel-capture-event");
  };

  return (
    <Flex direction="column" gap="md">
      <Carousel maw={400} mx="" withIndicators height={200}></Carousel>
      <Center>
        <Text>{captureMessage}</Text>
      </Center>
      <Progress value={captureProgress} />
      <Button color="red" onClick={handleCancelCapture}>
        Cancel
      </Button>
    </Flex>
  );
};

export default CaptureForm;
