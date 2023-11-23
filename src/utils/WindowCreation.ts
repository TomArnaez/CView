import { Chart, commands } from "../bindings"
import { Window } from "@tauri-apps/api/window";

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export const createChartWindow = (chart: Chart, imageIdx: number, stackIdx: number): Window => {
    const label = chart + "-Image" + imageIdx + "-Stack" + stackIdx;
 
    const chartWindow = new Window(label, {
        url: "src/windows/profilechart.html",
      });

      chartWindow.once("tauri://created", async function () {
        const mainWindow = Window.getByLabel("main");

        await mainWindow?.onCloseRequested(async () => {
            chartWindow.close();
        });

        chartWindow.show();
        chartWindow.setTitle(label);

        console.log(imageIdx, stackIdx, chartWindow.label);

        // TODO: Have window signal when it's completed its useEffect setup so we don't have to sleep
        await sleep(500);

        commands.subscribeChart(
            chartWindow.label,
            imageIdx,
            stackIdx,
            chart
        );
      });

      return chartWindow;
}