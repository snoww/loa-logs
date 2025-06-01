import * as echarts from "echarts/core";

import { LineChart, ScatterChart } from "echarts/charts";

import {
  DatasetComponent,
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  MarkPointComponent,
  TitleComponent,
  ToolboxComponent,
  TooltipComponent
} from "echarts/components";

import { SVGRenderer } from "echarts/renderers";

echarts.use([
  TitleComponent,
  TooltipComponent,
  DataZoomComponent,
  GridComponent,
  ToolboxComponent,
  DatasetComponent,
  LegendComponent,
  MarkPointComponent,
  SVGRenderer,
  LineChart,
  ScatterChart
]);

export type EChartsOptions = echarts.EChartsCoreOption;
export type EChartsTheme = string | object;
export type EChartsRenderer = "canvas" | "svg";
export type ChartOptions = {
  theme?: EChartsTheme;
  renderer?: EChartsRenderer;
  options: EChartsOptions;
};

const DEFAULT_OPTIONS: Partial<ChartOptions> = {
  theme: undefined,
  renderer: "svg"
};

export function chartable(element: HTMLElement, options: EChartsOptions) {
  const { theme, renderer } = {
    ...DEFAULT_OPTIONS
  };
  echarts.getInstanceByDom(element)?.dispose();
  const echartsInstance = echarts.init(element, theme, { renderer });
  echartsInstance.setOption(options, { notMerge: true });
  function handleResize() {
    echartsInstance.resize();
  }
  window.addEventListener("resize", handleResize);
  return {
    echartsInstance,
    destroy() {
      echartsInstance.dispose();
      window.removeEventListener("resize", handleResize);
    },
    update(newOptions: EChartsOptions) {
      echartsInstance.setOption({
        ...newOptions
      });
    }
  };
}

export const defaultOptions: EChartsOptions = {
  textStyle: {
    fontFamily: "Inter Variable"
  },
  grid: {
    left: "2%",
    right: "5%",
    bottom: "16%",
    top: "15%",
    containLabel: true
  },
  tooltip: {
    trigger: "axis"
  },
  toolbox: {
    feature: {
      restore: {}
    }
  },
  dataZoom: [
    {
      type: "inside",
      throttle: 50
    },
    {
      type: "slider",
      fillerColor: "rgba(80,80,80,.5)",
      borderColor: "rgba(80,80,80,.5)",
      handleStyle: {
        color: "rgba(80,80,80,.5)"
      },
      moveHandleStyle: {
        color: "rgba(136,136,136)"
      }
    }
  ]
};
