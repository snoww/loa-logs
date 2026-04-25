<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.43.0-Nineveh Beta 10 - April 24th, 2026
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

#### NEW FEATURES

initial rdps patch (big thanks to poont, and molden for nineveh)

- added ndps, rdps, rcon% columns
    - **ndps**: neutral damage = damage - damage from party buffs
    - **rdps**: raid damage = damage - damage from party buffs + damage given by your buffs
    - rcon%:
        - **dps**: contribution % from all party members' buffs, i.e. how much damage you did was from the party (dps + support buffs)
        - **support**: how much damage you contributed to the party, from buffs, bracelet, cards, drops of ether, etc.
    - reminder on udps: udps is derived from combat analyzer data, which ONLY includes support buffs/debuffs, no cards, bracelet, drops, etc., while rdps is calculated from player stats accounts for almost all buffs/debuffs that are applied to the player
    - standing striker/luminary uptimes no longer show uptime at max stacks due to usage for rdps

**calculating rdps is quite cpu intensive, if you have performance issues such as fps drops in game, please report your pc specs and negative experiences in ramen shop**

#### BUG FIXES & IMPROVEMENTS
- fixed random nineveh server select message when using mini meter
- maybe fixed ban message showing up in random dungeons


**Older Changelogs**: [link](https://github.com/snoww/loa-logs/releases/tag/v1.40.4)
