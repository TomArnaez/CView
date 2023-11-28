import "./App.css";
import {
  AppShell,
  Menu,
  Burger,
  SegmentedControl,
  Modal,
  Flex,
  ActionIcon,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import {
  FaMousePointer,
  FaLongArrowAltRight,
  FaRegFileImage,
  FaSave,
  FaRegSquare,
  FaArrowLeft,
  FaArrowRight,
} from "react-icons/fa";
import { useState, useEffect, useCallback } from "react";
import {
  commands,
  events,
  AdvancedCapture,
  LiveCapture,
  CaptureManagerEventPayload,
  CaptureProgress,
} from "./bindings";
import CaptureForm from "./components/CaptureForm";
import { ImageList } from "./components/ImageList";
import { listen } from "@tauri-apps/api/event";
import { CaptureSettings } from "./components/CaptureSettings";
import { Mode } from "./types/draw";
import { Viewer } from "./components/Viewer/Viewer";
import { useImageStore } from "./stores/imageStore";
import { GeneralSettingsForm } from "./components/GeneralSettingsForm";
import useDetectorListener from "./hooks/useDetectorListener";
import CaptureButton from "./components/CaptureButton";
import StreamButton from "./components/StreamButton";
import ImageListRadix from "./components/ImageList/ImageListRadix";
import { useAppSettingsStore } from "./stores/appSettingsStore";

function App() {
  useDetectorListener();
  const [captureProgressModalOpened, setCaptureProgressModalOpened] =
    useState(false);
  const [captureSettingsModalOpened, setCaptureSettingsModalOpened] =
    useState(false);
  const [generalSettingsOpened, generalSettingsHandlers] = useDisclosure(false);
  const [mapGenerationFormOpened, mapGenerationFormHandlers] =
    useDisclosure(false);
  const {
    setStreaming,
    currentImageIdx,
    currentStackIdx,
    incrementImage,
    decrementImage,
    setStack,
    updateStacks,
  } = useImageStore((state) => ({
    setStreaming: state.setStreaming,
    currentImageIdx: state.currentImageIndex,
    currentStackIdx: state.currentStackIndex,
    incrementImage: state.incrementImage,
    decrementImage: state.decrementImage,
    setStack: state.setStack,
    setImage: state.setImage,
    updateStacks: state.updateStacks,
  }));

  const { autoSaveCaptures } = useAppSettingsStore((state) => ({
    autoSaveCaptures: state.autoSaveCaptures,
  }));

  const [captureManagerInfo, setCaptureManagerInfo] =
    useState<CaptureManagerEventPayload>({
      status: "DetectorDisconnected",
      dark_maps: [],
    });
  const [captureProgress, setCaptureProgress] =
    useState<CaptureProgress | null>(null);
  const [drawMode, setDrawMode] = useState<Mode>(Mode.SelectionMode);
  const [live, setLive] = useState<boolean>(false);

  const handleUserKeyPress = useCallback(
    (event: KeyboardEvent) => {
      const { key, ctrlKey } = event;

      switch (key) {
        case "ArrowRight":
          incrementImage(1);
          break;
        case "ArrowLeft":
          decrementImage(1);
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
    [decrementImage, incrementImage]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleUserKeyPress);

    events.lineProfileEvent.listen((e) => {});

    listen("image-state-event", (e: any) => {
      updateStacks(e.payload);
    });

    events.captureProgressEvent.listen(async (e) => {
      setCaptureProgress(e.payload);
    });

    events.captureManagerEvent.listen(async (e) => {
      setCaptureManagerInfo(e.payload);
    });

    return () => {
      window.removeEventListener("keydown", handleUserKeyPress);
    };
  }, [handleUserKeyPress, updateStacks]);

  const handleSaveImage = async () => {
    await commands.saveImage(currentImageIdx, currentStackIdx);
  };

  const handleSaveStack = async () => {
    await commands.saveStack(currentStackIdx);
  };

  const handleCapture = async (capture: AdvancedCapture) => {
    setStreaming(true);
    setCaptureSettingsModalOpened(false);
    await commands.runCapture(capture, autoSaveCaptures);
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

  const handleOpenImages = async () => {
    await commands.openImages();
    setStack(0);
  };

  const handleGenerateDarkMaps = async () => {
    await commands.generateDarkMaps();
  };

  const handleGenerationDefectMap = async () => {
    await commands.generateDefectMap();
  };

  return (
    <>
      <Modal
        zIndex={11}
        opened={generalSettingsOpened}
        onClose={generalSettingsHandlers.close}
        centered
      >
        <GeneralSettingsForm />
      </Modal>
      <Modal
        zIndex={11}
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

                <Menu.Divider />

                <Menu.Item onClick={generalSettingsHandlers.open}>
                  Setttings
                </Menu.Item>
              </Menu.Dropdown>
            </Menu>
            <CaptureButton onClick={handleAdvancedCapture} />
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
                  label: <FaRegSquare />,
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
            <StreamButton />
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
            <ActionIcon onClick={() => decrementImage(1)} variant="transparent">
              <FaArrowLeft />
            </ActionIcon>
            <Viewer drawMode={drawMode} interaction={false} />
            <ActionIcon onClick={() => incrementImage(1)} variant="transparent">
              <FaArrowRight />
            </ActionIcon>
          </Flex>
        </AppShell.Main>
      </AppShell>
    </>
  );
}

export default App;
