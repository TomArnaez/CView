import "./App.css";
import {
  AppShell,
  Menu,
  Burger,
  Button,
  SegmentedControl,
  Modal,
  Flex,
  ActionIcon,
  Progress,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import {
  FaBorderStyle,
  FaMousePointer,
  FaLongArrowAltRight,
  FaRegFileImage,
  FaSave,
} from "react-icons/fa";
import { useState, useEffect, useCallback } from "react";
import {
  commands,
  Annotation,
  events,
  DetectorState,
  AdvancedCapture,
  LiveCapture,
  AppData,
  CaptureManagerEventPayload,
  CaptureProgress,
  CaptureManagerStatus,
} from "./bindings";
import CaptureForm from "./components/CaptureForm";
import { ImageList } from "./components/ImageList";
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { CaptureSettings } from "./components/CaptureSettings";
import { camelCaseToWords, convert14BArrayToRGBA, streamWorker } from "./utils";
import { invoke } from "@tauri-apps/api/primitives";
import { Mode } from "./types/draw";
import { Viewer } from "./components/Viewer/Viewer";
import { useImageStore } from "./stores/ImageStore";
import classes from "./css/Button.module.css";

function App() {
  const [captureProgressModalOpened, setCaptureProgressModalOpened] =
    useState(false);
  const [captureSettingsModalOpened, setCaptureSettingsModalOpened] =
    useState(false);
  const [generalSettingsOpened, generalSettingsHandlers] = useDisclosure(false);
  const {
    imageStacks,
    currentImageIdx,
    currentStackIdx,
    incrementImage,
    decrementImage,
    setStack,
    updateStacks,
  } = useImageStore((state) => ({
    imageStacks: state.imageStacks,
    currentImageIdx: state.currentImageIndex,
    currentStackIdx: state.currentStackIndex,
    incrementImage: state.incrementImage,
    decrementImage: state.decrementImage,
    setStack: state.setStack,
    setImage: state.setImage,
    updateStacks: state.updateStacks,
  }));

  const [captureManagerInfo, setCaptureManagerInfo] =
    useState<CaptureManagerEventPayload>({
      status: "DetectorDisconnected",
      dark_maps: [],
    });
  const [streaming, setStreaming] = useState<boolean>(false);
  const [unlistenStreamEventState, setUnlistenStreamEventState] =
    useState<UnlistenFn | null>(null);
  const [captureProgress, setCaptureProgress] = useState<CaptureProgress | null>(null);
  const [drawMode, setDrawMode] = useState<Mode>(Mode.SelectionMode);
  const [imageCanvas, setImageCanvas] = useState<HTMLCanvasElement | null>(
    null
  );
  const [live, setLive] = useState<boolean>(false);

  const handleUserKeyPress = useCallback(
    (event: KeyboardEvent) => {
      const { key, ctrlKey } = event;
      const { histogramEquilization } = commands;

      switch (key) {
        case "ArrowRight":
          incrementImage(1);
          break;
        case "ArrowLeft":
          decrementImage(1);
          break;
        case "i":
          invert(false, currentImageIdx, currentStackIdx);
          break;
        case "r":
          histogramEquilization(currentImageIdx, currentStackIdx);
          break;
        case "s":
          if (ctrlKey) {
            handleSaveImage();
          } else {
            setCaptureSettingsModalOpened(true);
          }
          break;
        default:
      }
    },
    [currentImageIdx, currentStackIdx, decrementImage, incrementImage]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleUserKeyPress);

    events.lineProfileEvent.listen((e) => {
    });

    listen("image-state-event", (e: any) => {
      updateStacks(e.payload);
    });

    events.captureProgressEvent.listen(async (e) => {
      setCaptureProgress(e.payload)
    });

    events.captureManagerEvent.listen(async (e) => {
      console.log(e.payload)
      setCaptureManagerInfo(e.payload);
    });

    return () => {
      window.removeEventListener("keydown", handleUserKeyPress);
    };
  }, []);

  const refreshImage = async (): Promise<void> => {
    await refreshCanvas(
      convert14BArrayToRGBA(await fetchImageData(), 1031, 1536)
    );
  };

  useEffect(() => {
    if (imageStacks.length > 0) refreshImage();
  }, [currentImageIdx, currentStackIdx, imageStacks]);

  const fetchImageData = async (): Promise<Uint16Array> => {
    const data = new Uint16Array(
      await invoke("get_image_binary", {
        imageIdx: currentImageIdx,
        stackIdx: currentStackIdx,
        resize: null,
      })
    );

    return data;
  };

  const refreshCanvas = async (data: Uint8Array) => {
    if (data == null) return;

    const width = 1031;
    const height = 1536;
    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;

    const ctx = canvas.getContext("2d");
    if (ctx != null) {
      const imageData = ctx.createImageData(width, height);
      imageData.data.set(data);
      ctx.putImageData(imageData, 0, 0);
    }

    setImageCanvas(canvas);
  };

  const handleSaveImage = async () => {
    await commands.saveImage(currentImageIdx, currentStackIdx);
  };

  const handleSaveStack = async () => {
    await commands.saveStack(currentStackIdx);
  };

  const listenStreamEvent = async (): Promise<UnlistenFn> => {
    return events.streamCaptureEvent.listen(async () => {
      const data = new Uint16Array(await invoke("read_stream_buffer", {}));
      if (data.length != 0) {
        const width = 1031;
        const height = 1536;
        refreshCanvas(convert14BArrayToRGBA(data, width, height));
      }
    });
  };

  const handleCapture = async (capture: AdvancedCapture) => {
    setCaptureSettingsModalOpened(false);
    await commands.runCapture(capture, true);
  };

  const handleGoLive = async () => {
    let unlisten = await listenStreamEvent();
    setUnlistenStreamEventState(await listenStreamEvent());
    setStreaming(true);
    const capture: LiveCapture = {
      exp_time: 100,
      type: "LiveCapture",
    };
    await commands.runCapture(capture, false);
  };

  const handleStopLive = async () => {
    await commands.stopCapture();
    if (unlistenStreamEventState != null) {
      setUnlistenStreamEventState(null);
    }
    function delay(ms) {
      return new Promise((resolve) => setTimeout(resolve, ms));
    }

    // TODO: Find a bettr way to clear stream event
    await delay(500);
    setImageCanvas(null);
    setStreaming(false);
  };

  const handleAdvancedCapture = async () => {
    if (captureManagerInfo.status == "DarkMapsRequired") {
      commands.generateDarkMaps();
    } else if (captureManagerInfo.status == "DefectMapsRequired") {
      commands.generateDefectMap();
    } else if (captureManagerInfo.status == "Available") {
      setCaptureSettingsModalOpened(true);
    }
  };

  const handleChangeStack = async (index: number) => {
    setStack(index);
    console.log("changing stack");
  };

  const handleOpenImages = async () => {
    await commands.openImages();
    setStack(0);
    refreshImage(await fetchImageData());
  };

  const handleGenerateDarkMaps = async () => {
    await commands.generateDarkMaps();
  };

  const handleGenerationDefectMap = async () => {
    await commands.generateDefectMap();
  };

  function isCapturingStatus(status: CaptureManagerStatus): status is { Capturing: AdvancedCapture } {
    return typeof status === 'object' && 'Capturing' in status;
  }
  

  return (
    <>
      <Modal
        opened={generalSettingsOpened}
        onClose={generalSettingsHandlers.close}
        centered
      ></Modal>
      <Modal
        centered
        opened={captureProgressModalOpened}
        closeOnEscape={false}
        closeOnClickOutside={false}
        onClose={() => setCaptureProgressModalOpened(false)}
      >
        <CaptureForm setFormOpen={setCaptureProgressModalOpened} />
      </Modal>
      <Modal
        centered
        withinPortal={true}
        opened={captureSettingsModalOpened}
        onClose={() => setCaptureSettingsModalOpened(false)}
      >
        <CaptureSettings
          darkMapExps={captureManagerInfo.dark_maps}
          startCapture={handleCapture}
        />
      </Modal>
      <AppShell
        style={{
          width: "100vw",
          height: "100vh",
        }}
        header={{ height: 100 }}
        navbar={{ width: 200, breakpoint: "sm" }}
      >
        <AppShell.Navbar>
          <ImageList />
        </AppShell.Navbar>

        <AppShell.Header p="xs" style={{ display: "flex" }} zIndex={999}>
          <div style={{ flex: 1, display: "flex", alignItems: "center" }}>
            <Menu shadow="md" width={200}>
              <Menu.Target>
                <Burger opened={false}></Burger>
              </Menu.Target>

              <Menu.Dropdown>
                <Menu.Label>File</Menu.Label>
                <Menu.Item
                  onClick={handleOpenImages}
                  icon={<FaRegFileImage size={14} />}
                >
                  Open
                </Menu.Item>
                <Menu.Item
                  onClick={handleSaveImage}
                  icon={<FaSave size={14} />}
                >
                  Save Current Image
                </Menu.Item>
                <Menu.Item
                  onClick={handleSaveStack}
                  icon={<FaSave size={14} />}
                >
                  Save Current Stack
                </Menu.Item>

                <Menu.Divider />

                <Menu.Label>Capture</Menu.Label>
                <Menu.Item
                  onClick={() => setCaptureSettingsModalOpened(true)}
                  icon={<FaRegFileImage size={14} />}
                >
                  Capture Settings
                </Menu.Item>
                <Menu.Item
                  onClick={handleGenerateDarkMaps}
                  icon={<FaRegFileImage size={14} />}
                >
                  Generate Dark Maps
                </Menu.Item>
                <Menu.Item
                  onClick={handleGenerationDefectMap}
                  icon={<FaRegFileImage size={14} />}
                >
                  Generate Defect Map
                </Menu.Item>

                <Menu.Divider />

                <Menu.Item onClick={generalSettingsHandlers.open}>
                  Setttings
                </Menu.Item>
              </Menu.Dropdown>
            </Menu>
            <Button
              style={{
                height: "100%",
              }}
              variant="filled"
              fullWidth
              color={captureManagerInfo.status == "Available" ? "blue" : "red"}
              disabled={captureManagerInfo.status == "DetectorDisconnected"}
              onClick={handleAdvancedCapture}
            >
              {captureManagerInfo.status == "DarkMapsRequired" && (
                <> Generate Dark Maps </>
              )}
              {captureManagerInfo.status == "DefectMapsRequired" && (
                <> Generate Defect Map </>
              )}
              {captureManagerInfo.status == "Available" && (
                <> Run Advanced Capture </>
              )}
              {isCapturingStatus(captureManagerInfo.status) && (
                <> Running {camelCaseToWords(captureManagerInfo.status.Capturing.type)}
                <br/>
                <br/> 
                {captureProgress?.message} 
                </>
              )}
            </Button>
          </div>

          <div
            style={{
              flex: 1,
              display: "flex",
              justifyContent: "center",
              alignItems: "center",
            }}
          >
            <SegmentedControl
              value={drawMode}
              size="lg"
              color="blue"
              onChange={(value: Mode) => setDrawMode(value)}
              data={[
                {
                  value: Mode.SelectionMode,
                  label: <FaMousePointer />,
                },
                {
                  value: Mode.RectangleMode,
                  label: <FaBorderStyle />,
                },
                {
                  value: Mode.LineMode,
                  label: <FaLongArrowAltRight />,
                },
              ]}
            />
          </div>

          <div
            style={{
              flex: 1,
              display: "flex",
              justifyContent: "flex-end",
              alignItems: "center",
            }}
          >
            <Button
              style={{
                height: "100%",
              }}
              variant="filled"
              color={live ? "red" : "blue"}
              disabled={
                captureManagerInfo.status == "DetectorDisconnected" ||
                captureManagerInfo.status == "DarkMapsRequired" ||
                captureManagerInfo.status == "DefectMapsRequired"
              }
              onClick={() => {
                live ? handleStopLive() : handleGoLive();
                setLive(!live);
              }}
              fullWidth
            >
              <div className={classes.label}>
                {live ? "Stop Live" : "Go Live"}
              </div>
            </Button>
          </div>
        </AppShell.Header>

        <AppShell.Main style={{ width: "100%", height: "100%" }}>
          <Flex
            style={{ height: "100%", width: "100%", backgroundColor: "red" }}
            mih={50}
            bg="white"
            gap="md"
            justify="flex-start"
            align="center"
            direction="row"
            wrap="nowrap"
          >
            {imageCanvas && (
              <Viewer
                drawMode={drawMode}
                refreshImage={refreshImage}
                imageCanvas={imageCanvas}
                interaction={!streaming}
              />
            )}
          </Flex>
        </AppShell.Main>
      </AppShell>
    </>
  );
}

export default App;
