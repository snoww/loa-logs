<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { appWindow } from "@tauri-apps/api/window";
    import { Tooltip } from 'flowbite-svelte';

    export let encounterDuration: string;
    export let totalDamageDealt: number;
    export let dps: number;

    async function openLogWindow() {
        await invoke("open_most_recent_encounter");
    }
    async function openSettingsWindow() {
        await invoke("open_url", { url: "settings" });
    }
</script>

<div class="px-2 py-1 bg-zinc-900/[.6] text-sm fixed top-0 left-0 h-7 w-full z-50" id="header">
    <div data-tauri-drag-region class="flex justify-between">
        <div data-tauri-drag-region class="flex space-x-2">
            <div data-tauri-drag-region>
                {encounterDuration}
            </div>
            <div data-tauri-drag-region class="flex space-x-1 tracking-tighter text-gray-400">
                <div data-tauri-drag-region>
                    Total DMG
                </div>
                <div data-tauri-drag-region>
                    {totalDamageDealt.toLocaleString()}
                </div>
            </div>
            <div class="flex space-x-1 tracking-tighter text-gray-400">
                <div data-tauri-drag-region>
                    Total DPS
                </div>
                <div data-tauri-drag-region >
                    {dps.toLocaleString("en", { minimumFractionDigits: 0, maximumFractionDigits: 0 })}
                </div>
            </div>
        </div>
        <div data-tauri-drag-region class="flex space-x-1 items-center">
            <button class="" on:click={openLogWindow}>
                <div>
                    <svg class="w-5 h-5 fill-gray-400 hover:fill-gray-50" xmlns="http://www.w3.org/2000/svg" height="48" viewBox="0 96 960 960" width="48"><path d="M475.946 956.5Q316 956.5 206.545 845.323 97.091 734.147 98.5 574.5H190q1.152 120.8 83.513 205.65Q355.873 865 475.825 865q120.675 0 206.425-85.834Q768 693.331 768 571.184q0-119.148-85.937-201.666Q596.127 287 475.5 287q-59.675 0-112.087 24Q311 335 270.5 376H353v70.5H129.5V225H198v94.5q54-58 125.194-91.5 71.195-33.5 152.456-33.5 78.85 0 148.632 30.13 69.782 30.13 122.49 81.511 52.709 51.381 82.718 120.054 30.01 68.673 30.01 147.739t-30.01 148.805q-30.009 69.739-82.5 121.75Q694.5 896.5 624.76 926.5q-69.74 30-148.814 30Zm123.554-214L448 592.565V375.5h68.5v187L650 692l-50.5 50.5Z"/></svg>
                </div>
                <Tooltip placement="bottom" defaultClass="bg-zinc-700 p-2 text-gray-300">
                    Open Recent
                </Tooltip>
            </button>
            <button>
                <div>
                    <svg class="w-5 h-5 fill-gray-400 hover:fill-gray-50" xmlns="http://www.w3.org/2000/svg" height="48" viewBox="0 96 960 960" width="48"><path d="M725.323 994.5q-55.323 0-95.073-39.971-39.75-39.972-39.75-94.794 0-7.018 1.5-17.693t4.5-20.396L331 667q-17.788 20.5-42.942 32.25Q262.903 711 236.486 711q-56.308 0-95.897-39.803Q101 631.395 101 575.676q0-55.301 39.672-94.988Q180.344 441 236.382 441q26.707 0 50.912 10.25Q311.5 461.5 331 481.5l265.5-152.573q-3-8.056-4.5-18.398-1.5-10.343-1.5-18.915 0-54.864 39.927-94.739Q670.353 157 725.677 157q55.323 0 95.073 39.746 39.75 39.747 39.75 95.048 0 55.718-39.672 95.462T724.867 427q-26.998 0-51.259-7.719T632.5 391L366 535.5q3 8.786 4.002 21.183 1.002 12.397 1.002 19.575 0 7.177-1.002 15.938Q369 600.957 366 610.941L632.5 759.5q16.636-17.562 39.866-26.281 23.231-8.719 52.648-8.719 56.308 0 95.897 39.725 39.589 39.726 39.589 95.334 0 55.608-39.927 95.274-39.926 39.667-95.25 39.667Zm.212-648q22.752 0 38.609-15.891Q780 314.717 780 291.965t-15.891-38.609Q748.217 237.5 725.465 237.5t-38.609 15.891Q671 269.283 671 292.035t15.891 38.609q15.892 15.856 38.644 15.856Zm-489.5 284q22.753 0 38.609-15.891 15.856-15.892 15.856-38.644t-15.891-38.609Q258.717 521.5 235.965 521.5q-22.753 0-38.609 15.891-15.856 15.892-15.856 38.644t15.891 38.609q15.892 15.856 38.644 15.856Zm489.5 283.5q22.752 0 38.609-15.891Q780 882.217 780 859.465t-15.891-38.609Q748.217 805 725.465 805t-38.609 15.891Q671 836.783 671 859.535t15.891 38.609Q702.783 914 725.535 914Zm-.035-622ZM236 576Zm489.5 283.5Z"/></svg>
                </div>
                <Tooltip placement="bottom" defaultClass="bg-zinc-700 p-2 text-gray-300">
                    Live Share (Soon™)
                </Tooltip>
            </button>
            <button on:click={openSettingsWindow}>
                <div>
                    <svg class="w-5 h-5 fill-gray-400 hover:fill-gray-50" xmlns="http://www.w3.org/2000/svg" height="48" viewBox="0 96 960 960" width="48"><path d="m369 996-21-132.5q-7.5-1.5-24.75-11.25T289 831l-123 57L52.5 687l113-82q-1-5.975-1.5-13.987-.5-8.013-.5-15.013 0-5.5.5-14.25t1.5-15.75l-113-83L166 264.5 292 321q9-7.5 25.25-16.75T348 290.5l21-136h222.5l21 134.5q13 5 29.75 14T670 321l125.5-56.5L908 463l-115.5 82.041q.5 6.959 1.75 15.209T795.5 576q0 7.5-1.25 15.425-1.25 7.925-1.75 14.657L907.5 687 794 888l-125-57.5q-9.971 8.864-25.236 18.182Q628.5 858 612.5 863.5l-21 132.5H369Zm108.231-292.5q52.805 0 90.037-37.275 37.232-37.276 37.232-90.25 0-52.975-37.268-90.225Q529.964 448.5 477 448.5q-53.5 0-90.5 37.275-37 37.276-37 90.25 0 52.975 37 90.225 37 37.25 90.731 37.25ZM477 646q-30.5 0-50.25-20.5T407 576q0-29 19.75-49.5T477 506q29 0 49.5 20.5T547 576q0 29-20.5 49.5T477 646Zm3-70.5Zm-35.651 329h70.085L530 793q33.585-8.066 63.585-24.443 30-16.377 53.915-43.557L754 771.5l30.5-58.256L692 646q3.5-18 6.75-35.209 3.25-17.21 3.25-34.91Q702 558 699.5 541t-7.5-35l93.5-67.5-31-58.5-106 46.5q-22.5-27-53.25-46t-65.25-23L516 247h-72l-12.5 110.535q-37 6.465-67 23.965t-55 45L206 380l-32 58.5 91 65.5q-4 18.5-7 36.319-3 17.819-3 35.151 0 18.03 2.75 36.28t6.75 35.75L174 713l31.859 58.5 103.641-46q25.5 26.5 56.25 43.75t65.75 25.25l12.849 110Z"/></svg>
                </div>
                <Tooltip placement="bottom" defaultClass="bg-zinc-700 p-2 text-gray-300">
                    Open Settings
                </Tooltip>
            </button>
            <button on:click={() => appWindow.hide()}>
                <div>
                    <svg class="w-5 h-5 fill-gray-400 hover:fill-gray-50" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <path d="M20 14H4v-4h16"/>
                    </svg>
                </div>
                <Tooltip placement="bottom" defaultClass="bg-zinc-700 p-2 text-gray-300">
                    Minimize
                </Tooltip>
            </button>
        </div>
    </div>
</div>