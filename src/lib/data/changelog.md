<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.43.0-Nineveh Beta 13 - April 30th, 2026
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

The LOA Logs beta uses a completely overhauled system for reading Lost Ark's networking data, giving us significantly more control over how we read and query game information. It finally fixes the issue where damage numbers would sometimes completely glitch out, and provides us the necessary infrastructure to implement rDPS in the future. By helping us test its stability, we can be confident that our new "Nineveh" packet library works correctly and efficiently across all platforms, including Linux!

### Note: ExitLag users must check the ExitLag Compatibility setting

#### read about rdps in depth [here](https://github.com/snoww/loa-logs/wiki/rDPS-and-nDPS-Explained)

**calculating rdps is quite cpu intensive, if you have performance issues such as fps drops in game, please report your pc specs and negative experiences in ramen shop**

#### BUG FIXES & IMPROVEMENTS

- fixed rdps not reinspecting players after wipes
- fixed crit dmg being undervalued in rdps
- added dark grenade section in logs for dark grenade rdps
