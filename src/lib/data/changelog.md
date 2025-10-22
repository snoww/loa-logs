<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.33.11 - October 22nd, 2025
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### P.S. If you see any of us in town or raid, feel free to drop an honor!

- **Snowving** on Luterra
- **Administrator** on Ratik
- **Poont** on Gienah

#### BUG FIXES AND IMPROVEMENTS

- adjusted log uploads for region merge
- another speculative fix for "start with windows" setting, toggle on/off to apply
- fixed some skill tripod groupings
- fixed hyper awakening damage being counted in the non-ha dmg part of T skill under buff breakdown (overall T% still correct)

---

### v1.33.10 - October 9th, 2025

#### BUG FIXES AND IMPROVEMENTS

- improved skill grouping for certain tripods of skills
- fixed manual upload button not working
- fixed share log button not working
- fixed issue when trying to open certain old logs

---

### v1.33.9 - October 9th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fix error popup when quitting meter

---

### v1.33.8 - October 7th, 2025

#### BUG FIXES AND IMPROVEMENTS

- another speculative for start with windows setting (toggle setting to activate)
- fixed tooltips staying on screen even after mouse leaving meter window
- fixed missing damage % setting in live meter

---

### v1.33.7 - September 21th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fixed support bdmg/bdps not showing when meter opened late

---

### v1.33.6 - September 21th, 2025

#### BUG FIXES AND IMPROVEMENTS

- support bdmg/bdps now shows in the bdps/udps column in overview
- fixed stagger not counting when target was not a boss

---

### v1.33.5 - September 20th, 2025

#### BUG FIXES AND IMPROVEMENTS

- fixed contribution % showing incorrect value
- fixed udmg and udps not available on live meter
- fixed moonfall one bubble not tracking bdmg and bdps (and others)
- fixed start with windows setting (toggle on and off to apply)

---

### v1.33.4 - September 19th, 2025

#### NEW FEATURES

- RDPS
  - pseudo rdps based off of in-game Combat Analyzer (thanks to research by molen and poont)
  - for DPS classes:
    - **uDMG, uDPS (unbuffed)**: shows damage and dps without SUPPORT buffs (AP, Brand, Identity, and T). This is what the in-game combat analyzer uses to calculate rdps for supports
  - for support classes:
    - **bDMG, bDPS (buffed)**: the amount of damage that the skill buffed.
    - **bD%**: shows the amount of damage the support contributed relative to the entire raid (in support's breakdown).
    - **DR**: shows the damage mitigated by the skill
  - column visibility can be adjusted in settings, for logs and meter
- Future statistics and percentiles will be based off pseudo rdps once enough data has been collected. Please upload all your logs after you update!

#### BUG FIXES AND IMPROVEMENTS

- fixed adrenaline crit rate buff missing
- fixed players missing in encounter preview

![udps](https://i.imgur.com/PpP8PlK.png)

![udps breakdown](https://i.imgur.com/bdlweJ5.png)

![bdps](https://i.imgur.com/IT2R9yl.png)

---

### v1.33.3 - September 17th, 2025

#### NEW FEATURES

- added new stagger column
  - applies to skills, skill breakdowns, and skill hits

![party_stagger](https://i.imgur.com/JOEoZdI.png)

![skill_breakdown_stagger](https://i.imgur.com/UG7Zt8H.png)

#### BUG FIXES AND IMPROVEMENTS

- fix crashes when settings was malformed

---

### v1.33.2 - September 17th, 2025

#### Known Issues

- meter window position might be reset after update

#### NEW FEATURES

- updated meter for assault raids
- added skill names for paradise items

#### BUG FIXES AND IMPROVEMENTS

- improved meter backend
- fixed uploading not working
- fixed buffs not showing for older logs

---

**Older Changelogs**: [link](https://github.com/snoww/loa-logs/releases/tag/v1.32.5)
