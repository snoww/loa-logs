<script lang="ts">
  
  let { progress = 100, text = "Loading..." } = $props();

</script>

<div
  data-tauri-drag-region
  class="loader-wrapper h-screen flex flex-col justify-center items-center gap-4">
  <div data-tauri-drag-region class="background-overlay"></div>

  <div class="progress-bar">
    <div class="progress-fill" style="width: {progress}%"></div>
    <span class="progress-text">{text}</span>
  </div>
  
</div>

<style>
    .loader-wrapper {
      height: 100vh;
      display: flex;
      flex-direction: column;
      justify-content: center;
      align-items: center;
      gap: 1rem;

      background-image: url('https://uwuowo.mathi.moe/i/artist_cry.png');
      background-size: cover;
      background-position: center;
      background-repeat: no-repeat;
      transition: all 1s ease;
  }

  .background-overlay {
    position: absolute;
    inset: 0;
    background-color: rgba(0, 0, 0, .8);
    z-index: 0;
  }

  .progress-bar {
    position: relative;
    width: 300px;
    height: 30px;
    border: 2px solid white;
    border-radius: 5px;
    overflow: hidden;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .progress-fill {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    background-color: #000000BB;
    transition: width 0.2s ease;
    z-index: 0;
  }

  .progress-text {
    position: relative;
    color: white;
    font-weight: bold;
    z-index: 1;
    text-align: center;
  }

</style>
