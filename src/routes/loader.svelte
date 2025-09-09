<script lang="ts">
    export let text: string = "Loading...";
    export let size: string = "1em";
    export let color: string = "#fff";
</script>

<div class="loader-wrapper h-screen bg-black flex justify-center">
    <div class="loader" style="width: {size}; height: {size}; color: {color};"></div>
    <p class="text-white">{text}</p>
</div>

<style>
    .loader-wrapper {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 3rem;
    }

    .loader {
        display: inline-block;
        color: inherit;
        font-size: 10px;
        width: 1em;
        height: 1em;
        border-radius: 50%;
        position: relative;
        text-indent: -9999em;
        animation: mulShdSpin 1.3s infinite linear;
        transform: translateZ(0);
    }

    @keyframes mulShdSpin {
        0%, 100% {
        box-shadow: 0 -3em 0 0.2em, 
                    2em -2em 0 0em, 3em 0 0 -1em, 
                    2em 2em 0 -1em, 0 3em 0 -1em, 
                    -2em 2em 0 -1em, -3em 0 0 -1em, 
                    -2em -2em 0 0;
    }
    12.5% {
        box-shadow: 0 -3em 0 0, 2em -2em 0 0.2em, 
                    3em 0 0 0, 2em 2em 0 -1em, 0 3em 0 -1em, 
                    -2em 2em 0 -1em, -3em 0 0 -1em, 
                    -2em -2em 0 -1em;
    }
    25% {
        box-shadow: 0 -3em 0 -0.5em, 
                    2em -2em 0 0, 3em 0 0 0.2em, 
                    2em 2em 0 0, 0 3em 0 -1em, 
                    -2em 2em 0 -1em, -3em 0 0 -1em, 
                    -2em -2em 0 -1em;
    }
    37.5% {
        box-shadow: 0 -3em 0 -1em, 2em -2em 0 -1em,
                    3em 0em 0 0, 2em 2em 0 0.2em, 0 3em 0 0em, 
                    -2em 2em 0 -1em, -3em 0em 0 -1em, -2em -2em 0 -1em;
    }
    50% {
        box-shadow: 0 -3em 0 -1em, 2em -2em 0 -1em,
                    3em 0 0 -1em, 2em 2em 0 0em, 0 3em 0 0.2em, 
                    -2em 2em 0 0, -3em 0em 0 -1em, -2em -2em 0 -1em;
    }
    62.5% {
        box-shadow: 0 -3em 0 -1em, 2em -2em 0 -1em,
                    3em 0 0 -1em, 2em 2em 0 -1em, 0 3em 0 0, 
                    -2em 2em 0 0.2em, -3em 0 0 0, -2em -2em 0 -1em;
    }
    75% {
        box-shadow: 0em -3em 0 -1em, 2em -2em 0 -1em, 
                    3em 0em 0 -1em, 2em 2em 0 -1em, 0 3em 0 -1em, 
                    -2em 2em 0 0, -3em 0em 0 0.2em, -2em -2em 0 0;
    }
    87.5% {
        box-shadow: 0em -3em 0 0, 2em -2em 0 -1em, 
                    3em 0 0 -1em, 2em 2em 0 -1em, 0 3em 0 -1em, 
                    -2em 2em 0 0, -3em 0em 0 0, -2em -2em 0 0.2em;
    }
    }
</style>