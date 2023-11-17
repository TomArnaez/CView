         // This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

         export const commands = {
async generateDarkMaps() : Promise<__Result__<null, "DetectorDisconnected" | "DetectorInUse" | { File2Error: CorrectionError } | { SLError: InternalSLError } | "Unknown">> {
try {
    return { status: "ok", data: await TAURI_INVOKE("plugin:tauri-specta|generate_dark_maps") };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async runCapture(capture: AdvancedCapture) : Promise<__Result__<null, "DetectorDisconnected" | "DetectorInUse" | { File2Error: CorrectionError } | { SLError: InternalSLError } | "Unknown">> {
try {
    return { status: "ok", data: await TAURI_INVOKE("plugin:tauri-specta|run_capture", { capture }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async stopCapture() : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|stop_capture");
},
async startup() : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|startup");
},
async openImages() : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|open_images");
},
async saveImage(stackIndex: number, imageIndex: number) : Promise<__Result__<null, null>> {
try {
    return { status: "ok", data: await TAURI_INVOKE("plugin:tauri-specta|save_image", { stackIndex, imageIndex }) };
} catch (e) {
    if(e instanceof Error) throw e;
    else return { status: "error", error: e  as any };
}
},
async saveStack(stackIdx: number) : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|save_stack", { stackIdx });
},
async histogramEquilization(imageIdx: number, stackIdx: number) : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|histogram_equilization", { imageIdx, stackIdx });
},
async getPixelValue(x: number, y: number, stackIdx: number, imageIdx: number) : Promise<number | null> {
return await TAURI_INVOKE("plugin:tauri-specta|get_pixel_value", { x, y, stackIdx, imageIdx });
},
async updateRoi(annotation: Annotation, imageIdx: number, stackIdx: number) : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|update_roi", { annotation, imageIdx, stackIdx });
},
async invertColours(imageIdx: number, stackIdx: number) : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|invert_colours", { imageIdx, stackIdx });
},
async rotate(imageIdx: number, stackIdx: number, rotateLeft: boolean) : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|rotate", { imageIdx, stackIdx, rotateLeft });
},
async subscribeChart(label: string, imageIdx: number, stackIdx: number, chartType: Chart) : Promise<null> {
return await TAURI_INVOKE("plugin:tauri-specta|subscribe_chart", { label, imageIdx, stackIdx, chartType });
}
}

export const events = __makeEvents__<{
streamCaptureEvent: StreamCaptureEvent,
captureProgressEvent: CaptureProgressEvent,
cancelCaptureEvent: CancelCaptureEvent,
appDataEvent: AppDataEvent,
captureManagerEvent: CaptureManagerEvent,
imageStateEvent: ImageStateEvent,
lineProfileEvent: LineProfileEvent,
histogramEvent: HistogramEvent
}>({
streamCaptureEvent: "plugin:tauri-specta:stream-capture-event",
captureProgressEvent: "plugin:tauri-specta:capture-progress-event",
cancelCaptureEvent: "plugin:tauri-specta:cancel-capture-event",
appDataEvent: "plugin:tauri-specta:app-data-event",
captureManagerEvent: "plugin:tauri-specta:capture-manager-event",
imageStateEvent: "plugin:tauri-specta:image-state-event",
lineProfileEvent: "plugin:tauri-specta:line-profile-event",
histogramEvent: "plugin:tauri-specta:histogram-event"
})

/** user-defined types **/

export type AdvancedCapture = ({ type: "SmartCapture" } & SmartCapture) | ({ type: "SignalAccumulationCapture" } & SignalAccumulationCapture) | ({ type: "MultiCapture" } & MultiCapture) | ({ type: "DarkMapCapture" } & DarkMapCapture) | ({ type: "LiveCapture" } & LiveCapture)
export type Annotation = { Rect: Rect } | { Line: Line }
export type AppData = { dark_maps_files: { [key in number]: string }; defect_map: string | null }
export type AppDataEvent = AppData
export type BinningModesRS = RemoteBinningModes
export type CancelCaptureEvent = []
export type CaptureManagerEvent = CaptureManagerInfo
export type CaptureManagerInfo = { detector_info: DetectorInfo | null; status: CaptureManagerStatus }
export type CaptureManagerStatus = "Available" | "Capturing" | "DetectorDisconnected"
export type CaptureProgress = { message: string; current_step: number; total_steps: number }
export type CaptureProgressEvent = CaptureProgress
export type CaptureSetting = { exp_time: number; dds: boolean; full_well: FullWellModesRS; binning_mode: BinningModesRS; roi: number[] | null }
export type Chart = "Histogram" | "LineProfile"
export type CorrectionError = { SLError: InternalSLError } | { FileNotFound: string }
export type DarkMapCapture = { exp_times: number[]; frames_per_capture: number; type: "DarkMapCapture" }
export type DetectorInfo = { interface: string }
export type ExtraData = ({ type: "SmartCaptureData" } & SmartCaptureData) | ({ type: "SignalAccumulationData" } & SignalAccumulationData)
export type FullWellModesRS = { remote_ty: RemoteFullWellModes }
export type Histogram = { data: { [key in number]: number } }
export type HistogramEvent = Histogram
export type ImageHandler = { image_metadata: ImageMetadata; roi: Annotation | null; inverted_colours: boolean }
export type ImageMetadata = { capture_settings: CaptureSetting | null; date_created: string | null; extra_info: ExtraData | null }
export type ImageService = { image_stacks: ImageStack[] }
export type ImageStack = { timestamp: string | null; image_handlers: ImageHandler[]; capture: AdvancedCapture | null }
export type ImageStateEvent = ImageService
export type InternalSLError = string
export type Line = { start: Point; finish: Point }
export type LineProfileEvent = ([number, number])[]
export type LiveCapture = { exp_time: number; type: "LiveCapture" }
export type MultiCapture = { exp_times: number[]; frames_per_capture: number; type: "MultiCapture" }
export type Point = { x: number; y: number }
export type Rect = { width: number; height: number; pos: Point }
export type RemoteBinningModes = "BinningUnknown" | "x11" | "x22" | "x44"
export type RemoteFullWellModes = "High" | "Low" | "Enum"
export type SignalAccumulationCapture = { exp_times: number[]; frames_per_capture: number; type: "SignalAccumulationCapture" }
export type SignalAccumulationData = { accumulated_exp_time: number }
export type SmartCapture = { exp_times: number[]; frames_per_capture: number; window_size: number; median_filtered: boolean; type: "SmartCapture" }
export type SmartCaptureData = { signal_noise_ratio: number; background_rect: Rect; foreground_rect: Rect }
export type StreamCaptureEvent = []

/** tauri-specta globals **/

         import { invoke as TAURI_INVOKE } from "@tauri-apps/api/primitives";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type Window as __WebviewWindowHandle__ } from "@tauri-apps/api/window";

type __EventObj__<T> = {
  listen: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
  once: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
  emit: T extends null
    ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
    : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

type __Result__<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
  mappings: Record<keyof T, string>
) {
  return new Proxy(
    {} as unknown as {
      [K in keyof T]: __EventObj__<T[K]> & {
        (handle: __WebviewWindowHandle__): __EventObj__<T[K]>;
      };
    },
    {
      get: (_, event) => {
        const name = mappings[event as keyof T];

        return new Proxy((() => {}) as any, {
          apply: (_, __, [window]: [__WebviewWindowHandle__]) => ({
            listen: (arg: any) => window.listen(name, arg),
            once: (arg: any) => window.once(name, arg),
            emit: (arg: any) => window.emit(name, arg),
          }),
          get: (_, command: keyof __EventObj__<any>) => {
            switch (command) {
              case "listen":
                return (arg: any) => TAURI_API_EVENT.listen(name, arg);
              case "once":
                return (arg: any) => TAURI_API_EVENT.once(name, arg);
              case "emit":
                return (arg: any) => TAURI_API_EVENT.emit(name, arg);
            }
          },
        });
      },
    }
  );
}

     