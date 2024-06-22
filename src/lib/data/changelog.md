<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.11.1
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### NEW FEATURES (NOT REALLY NEW)
- counters tracking is now working
- death tracking also working

#### BUG FIXES AND IMPROVEMENTS
- fix icons for allied skills
- fix avele name (no longer the singer)
- (maybe) fixed thar special interaction showing as unknown
- removed g2 mirrors from hp bar tracking

**NOTE:** Please do note that this is a very barebones patch. Most of the meter features are **not** working properly due to missing packet structures. We are working hard to become fully independent of Herysia's meter-core, but this process is extremely difficult and will take a few weeks.

If you missed the news, please read the announcement in the discord channel. [#announcements - ramen shop](https://discord.gg/2rwTTKXRwu).

### Current Functionality
#### Working
- damage meter
- basic logs
- maybe some other stuff
#### Not Working
- party/self buff uptime tracking
- rdps
- shields
- stagger
- identity
- pretty much anything else not listed here


### v1.11.0
#### NEW FEATURES
- updated meter for echidna patch (special thanks to @PetAndMet and @faust)

---

### v1.10.8
#### NEW FEATURES
- **RDPS NOTE:** you must be updated to v1.10.8 or later to use rdps due to api backend change
#### BUG FIXES AND IMPROVEMENTS
- fix auto update causing meter to freeze up
- performance improvements
### v1.10.7
#### NEW FEATURES
- updates will be installed automatically at app start
- added changelog tab on sidebar to see recent release notes
#### BUG FIXES AND IMPROVEMENTS
- fixed issue where parties were missing players in buff tab
### v1.10.6
#### BUG FIXES AND IMPROVEMENTS
- removed south america in region detection
- remove brel gate 1 nightmare gehenna/helkasirs as bosses
- add voldis g2 100x split dragons as bosses
- add flag for manually saved encounters
### v1.10.5
#### BUG FIXES AND IMPROVEMENTS
- minor bug fixes and improvements
### v1.10.4
#### BUG FIXES
- fix cases where rdps marked invalid after player dies during raid
- more rdps improvements
- fix pants transcendence name missing
- preparing meter for general raid statistics
### v1.10.3
#### BUG FIXES
- fix meter not tracking in trial hanumatan
### v1.10.2
#### BUG FIXES
- fix azena icon showing as inanna
- minor fix to region detection
### v1.10.1
#### NEW FEATURES
- updated meter for trial hanumatan patch
#### BUG FIXES AND IMPROVEMENTS
- more improvements to rdps reliability
    - improved region detection for players that plays on multiple regions
    - fix cases where a single person's rdps was incorrectly calculated
    - fix cases where rdps will become inaccurate if players afk for too long (taking break between pulls)
- show ip address of network interfaces when _automatic network selection_ is disabled
- fix dead column still showing when players has respawned


**Older Changelogs**: [link](https://github.com/snoww/loa-logs/releases/tag/v1.9.10)

send bug reports/suggestions to [#loa-logs](https://discord.gg/sbSa3pkDF5)