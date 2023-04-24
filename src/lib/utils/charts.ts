import * as echarts from 'echarts/core';

import { LineChart, ScatterChart } from 'echarts/charts';

import {
    TitleComponent,
    TooltipComponent,
    DataZoomComponent,
    GridComponent,
    DatasetComponent,
    ToolboxComponent,
    LegendComponent,  
    MarkPointComponent,
} from 'echarts/components';

import { CanvasRenderer } from 'echarts/renderers';

echarts.use([
    TitleComponent, 
    TooltipComponent, 
    DataZoomComponent, 
    GridComponent, 
    ToolboxComponent, 
    DatasetComponent, 
    LegendComponent,
    MarkPointComponent,
    CanvasRenderer, 
    LineChart, 
    ScatterChart
]);

export type EChartsOptions = echarts.EChartsCoreOption
export type EChartsTheme = string | object
export type EChartsRenderer = 'canvas' | 'svg'
export type ChartOptions = {
    theme?: EChartsTheme
    renderer?: EChartsRenderer
    options: EChartsOptions
}

const DEFAULT_OPTIONS: Partial<ChartOptions> = {
    theme: undefined,
    renderer: 'canvas',
};


export function chartable(element: HTMLElement, options: EChartsOptions) {
    const { theme, renderer } = {
        ...DEFAULT_OPTIONS,
    };
    const echartsInstance = echarts.init(element, theme, { renderer });
    echartsInstance.setOption(options, { notMerge: true });
    function handleResize() {
        echartsInstance.resize();
    }
    window.addEventListener('resize', handleResize);
    return {
        destroy() {
            echartsInstance.dispose();
            window.removeEventListener('resize', handleResize);
        },
        update(newOptions: ChartOptions) {
            echartsInstance.setOption({
                ...options,
                ...newOptions.options,
            });
        },
    };
}

export const defaultOptions: EChartsOptions = {
    textStyle: {
        fontFamily: 'Inter',
    },
    grid: {
        left: '2%',
        right: '5%',
        bottom: '16%',
        top: '15%',
        containLabel: true
    },
    tooltip: {
        trigger: 'axis'
    },
    toolbox: {
        feature: {
            restore: {},
        }
    },
    dataZoom: [
        {
            type: 'inside',
            throttle: 50,
        },
        {
            type: 'slider',
            fillerColor: 'rgba(80,80,80,.5)',
            borderColor: "rgba(80,80,80,.5)",
            handleStyle: {
                color: 'rgba(80,80,80,.5)',
            },
            moveHandleStyle: {
                color: 'rgba(136,136,136)',
            },
        },
    ],
};