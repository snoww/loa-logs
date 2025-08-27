<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.32.5 - August 27th, 2025
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### NEW FEATURES
- update meter for weekly reset

#### BUG FIXES AND IMPROVEMENTS
- fix boss details missing in logs
- count skill effects to total casts
- minor improvements and fixes

---

### v1.32.3 - August 22th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fix self buffs showing wrong source skills
- fix buff summaries not excluding damage special skills

### v1.32.2 - August 20th, 2025

#### NEW FEATURES

- updated meter for valkyrie release (ty poon for the packets, niome for the class icon)
- changed dps column to performance column, shows support buffs for supports now (due to database changes, will only show for new logs)

#### BUG FIXES AND IMPROVEMENTS

- fix dps specs of supports counting towards support buffs
- fix paladin identity % not counting

---

### v1.31.9 - August 13th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fixed damage % not counting sidereal damage

### v1.31.8 - July 30th, 2025

#### NEW FEATURES

- added new column **_Cooldown Ratio %_** in log skill breakdowns
  - based off official meter in korea
  - shows percentage of time a skill was on cooldown for the duration of the encounter
  - _only works for local player_
  - will show in uploaded logs if you upload your pov

![cdr%](https://i.imgur.com/svbYR2h.png)

#### BUG FIXES AND IMPROVEMENTS

- fixed buff average %s not summing correctly
- fixed buff tab freezing in live meter

### v1.31.6 - July 27th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fixed artist ink marble identity duration increase not being tracked
- fixed gems for identity skills not being tracked
- improved summoner summon skill labeling (big thanks to poont)

### v1.31.5 - July 25th, 2025

#### NEW FEATURES

- miscellaneous encounters (cube, paradise, etc.) are hidden by default (settings > general > show raids only)
- exclude hyper awakening, transcendence, perfect blocks, paradise orbs, etc. from buffs and other modifier calculations (only for logs recorded after this update)

#### BUG FIXES AND IMPROVEMENTS

- fixed bard bubble %s not showing on tooltips
- fixed mini meter buffs

### v1.31.3 - July 24th, 2025

#### BUG FIXES AND IMPROVEMENTS

- updated classification of dps spec of supports
- fixed player not being recognized when loading in on a mount
- added missing gems for bard tempest, paladin divine wave, etc.
- added skill tracking for bard major/minor chords

### v1.31.2 - July 24th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fix berserker technique berserker classification
- fix parties being incorrect sometimes

### v1.31.0 - July 23rd, 2025

#### NEW FEATURES

- update meter for july paradise update (thanks poont for packets)
- added back details tab (enable in settings > general > show details)
  - shows your character's raw identity data
    ![details](https://i.imgur.com/ivGiQ4R.png)
- added combat power in tooltips

#### BUG FIXES AND IMPROVEMENTS

- improved skill grouping for some classes
- updated tripods display
- fixed boss hp chart for mordum g3 in certain cases
- show upload error message on upload failure

---

**Older Changelogs**: [link](https://github.com/snoww/loa-logs/releases/tag/v1.30.2)
