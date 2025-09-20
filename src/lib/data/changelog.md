<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.33.5 - September 20th, 2025
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### P.S. If you see any of us in town or raid, consider dropping an honor!

- Snowving on Luterra
- Administrator on Ratik
- Poont on Gienah

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
