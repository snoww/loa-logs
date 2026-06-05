<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.44.0-Nineveh Beta 26 - June 5th, 2026
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### P.S. If you see any of us in town or raid, feel free to drop an honor!

- **Snowving** on Luterra
- **Administrator** on Ratik
- **Poont** on Giena

#### About the Beta

The LOA Logs beta uses a completely overhauled system for reading Lost Ark's networking data, giving us significantly
more control over how we read and query game information. It finally fixes the issue where damage numbers would
sometimes completely glitch out, and provides us the necessary infrastructure to implement rDPS in the future. By
helping us test its stability, we can be confident that our new "Nineveh" packet library works correctly and efficiently
across all platforms, including Linux!

### Note: ExitLag users must check the ExitLag Compatibility setting

#### read about rdps in depth [here](https://github.com/snoww/loa-logs/wiki/rDPS-and-nDPS-Explained)

#### NEW FEATURES

- added rdps stats breakdown (thanks molen for the fancy graphs and breakdowns)
    - crit luck chart will be inaccurate the closer you are to 100% crit rate
    - stat dmg increase only works if you're not on low performance mode
    - older logs might look weird

![image](https://cdn.discordapp.com/attachments/1420857993863495701/1512237471578980573/image.png?ex=6a235c65&is=6a220ae5&hm=0228f88255bea48a433f1de7a43b69e7244859820873ba51d61b3afb93f16c7d&)

#### BUG FIXES AND IMPROVEMENTS

- maybe fix kaz g2 intermission ending late
- fixed unknown buff sources freezing logs