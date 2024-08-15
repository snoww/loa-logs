<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.14.0
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### NEW FEATURES
- update meter for calm before the storm update (thanks to @PoonT and @PetAndMet)

### v1.13.7
#### BUG FIXES AND IMPROVEMENTS
- fix using "delete all uncleared encounters" deleting the entire database

### v1.13.6
#### BUG FIXES AND IMPROVEMENTS
- fix class search not working
- fix migration errors for some users

### v1.13.5
#### NEW FEATURES
- greatly improved encounter searching and loading speed (@anyduck)
  - NOTE: launching meter after the update will require up to 30 seconds of database migrations
- allow changing number of rows displayed in the log table
- due to log sizes increasing because of every single skill cast being saved, future encounters will have fields compressed, greatly reducing database sizes for the future


### v1.13.4
#### BUG FIXES AND IMPROVEMENTS
- fixed certain skills not being detected in certain gates

**NOTE:** apologies for the broken updates, been super busy and not able to find time to work on the

### v1.13.3
#### BUG FIXES AND IMPROVEMENTS
- fixed parties not being determined correctly, causing buffs to show up incorrectly
- added back shields (shields might still be wrong for your own party in 8 man raids)

### v1.13.2
#### BUG FIXES AND IMPROVEMENTS
- actually fixed encounter not resetting after pulls
- fixed some buffs not showing correctly (e.g. harp of rhythm)

### v1.13.1
#### BUG FIXES AND IMPROVEMENTS
- fix encounter not resetting after pulls
- difficulty for guardians might be incorrect

### v1.13.0
#### NEW FEATURES
- updated meter for game qol update (thanks to @PoonT)
- shields are broken again


### v1.12.2
#### BUG FIXES AND IMPROVEMENTS
- fixed parties being recognized incorrectly, causing buffs to show up incorrectly or missing
- meter can now be opened before raid again still show buffs correctly (still try to open before character select tho)

### v1.12.1
#### NEW FEATURES
- shield tracking should be working again
- added the ability to differentiate bard brands (e.g. sound shock, harp of rhythm, stigma)

![brands](https://i.imgur.com/NQFBHlo.png)

#### BUG FIXES AND IMPROVEMENTS
- fixed app crashing when trying to view log with missing data
- added backwards compatibility to view old skill charts on older logs

### v1.12.0
#### NEW FEATURES
- updated meter for summer patch, thanks to @PetAndMet, @PoonT, @faust
- skill cast tracking
  - each damage tick of a skill is now tracked
  - you can look at the details of a skill cast by clicking on a skill icon in the skill cast graph

clicking on a skill icon in the graph brings up a tooltip that shows some basic info about the cast.

![graph tooltip](https://i.imgur.com/cS3OtwK.png)

scrolling down to the bottom of the page allows you to view the cast in more detail. where you are able to hover over columns for tooltips, and change the buff filtering to show different buffs applied during the damage tick. you can also click the arrows to look at the next/previous cast of the skill. it will also show rdps data when that is back and working again.

![skill details](https://i.imgur.com/kqvMxyR.png)


### v1.11.3
#### NEW FEATURES (NOT REALLY, AGAIN)
- buffs tracking should be working (huge thanks to @faust for getting these packets)
- raid difficulty labels are now working
- added warning in meter if meter is opened late

### v1.11.2
#### BUG FIXES AND IMPROVEMENTS
- thar name and icon now displays properly

### v1.11.1
#### NEW FEATURES (NOT REALLY NEW)
- counters tracking is now working
- death tracking also working

#### BUG FIXES AND IMPROVEMENTS
- fix icons for allied skills
- fix avele name (no longer the singer)
- (maybe) fixed thar special interaction showing as unknown
- removed g2 mirrors from hp bar tracking

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