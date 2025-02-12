<div class="rounded-md flex space-x-2 items-center">
  <div class="text-lg font-semibold text-white">
    v1.24.0
  </div>
  <div class="bg-accent-500 px-2 font-medium rounded-md text-white">
    New
  </div>
</div>

#### NEW FEATURES
- update meter for random feb patch

#### BUG FIXES AND IMPROVEMENTS

- fix some hyper awakenings counting towards buff uptime
- somewhat fixed act 2: brel g2 phase 1 hp bar amount
- add hp bar at time of wipe to wiped logs

### v1.23.1

#### BUG FIXES AND IMPROVEMENTS

- fix filtering of new brelshaza raid logs
- removed prokel's shadow as boss

### v1.23.0

#### NEW FEATURES

- update meter for act 2: brelshaza patch

#### BUG FIXES AND IMPROVEMENTS

- fix sharpshooter loyal companion classification

### v1.23.0

#### BUG FIXES AND IMPROVEMENTS

- fixed rare case of damage values being incorrect

### v1.22.3

#### NEW FEATURES

- added ark passive details on hover over character in logs

![ark passive details](https://i.imgur.com/nA1u5DG.png)

NOTE: this data is NOT always accurate. obvious invalid data (e.g. completely different spec) will not be shown. however more subtle issues/differences may be present. take the info with a grain of salt.

### v1.22.2

#### NEW FEATURES

- added new optional column _"Adjusted Crit Rate"_ for skill breakdowns (Settings > Logs > Scroll down to Skill Breakdown > Check "Adjusted Crit Rate")
    - calculates crit rate based on damage hits that do more than 5% of the average cast. this provides a more accurate crit rate for skills that have a big hit and multiple small hits or dots (e.g. Doomsday)
    - only shows on logs and not live meter

### v1.22.1

#### BUG FIXES AND IMPROVEMENTS

- fixed spec classification of destroyer and scouters

### v1.22.0

#### NEW FEATURES

- update meter for winter soloist patch

#### BUG FIXES AND IMPROVEMENTS

- fixed stabilized status buff showing when filtering "offensive buffs only"
- fixed punishing strike tripod counting as different skill
- fixed some errors in buff calculations
- fixed buff percentages in skill hit breakdowns
- grouped ark passive combat blessing/dance of passion buffs
- separated dark bomb and atropine buffs in self buffs

### v1.21.0

#### NEW FEATURES

- update meter for weekly maintenance

### v1.20.2

#### BUG FIXES AND IMPROVEMENTS

- fix certain damage skills not being correctly tracked due to new encryption
- fix source skills of certain buffs

### v1.20.1

#### BUG FIXES AND IMPROVEMENTS

- fix pants transcendence label
- fix battle items labels
- fix certain skills being separated

### v1.20.0

#### NEW FEATURES

- updated meter for thanksgiving patch

#### BUG FIXES AND IMPROVEMENTS

- added _estimate_ for stabilized status uptime (actual uptime will be equal or higher)

### v1.19.3

#### BUG FIXES AND IMPROVEMENTS

- updated icon to higher resolution (ty @raeinor)
- yet _another_ fix to start with windows setting (turn off and on again to reset it)
- fix gems showing wrong tier in certain cases
- fix serenade of amplification showing as 10% serenade
- fix scrapper spec classification

### v1.19.1

#### NEW FEATURES

- updated meter icon from suggestions

#### BUG FIXES AND IMPROVEMENTS

- _actually_ fixed start with windows setting (turn off and on again to reset it)
- remove movement speed buff from serenade percentage
- fix arcana sovereign and chancellor buffs being grouped together

### v1.19.0

#### NEW FEATURES

- updated meter for weekly reset
- count argeos ball towards dps to encourage ball hitting

#### BUG FIXES AND IMPROVEMENTS

- capture driver will be unloaded automatically when quitting meter
- capture driver will no longer prevent auto updater from updating for future updates after this one (hopefully)
- fixed start on boot option to use task scheduler (need to turn it off and back on for it to work)
- fix certain buff tooltips being incorrect
- moved ark passive self buffs into character breakdown

### v1.18.1

#### ADDITIONAL NOTES

- meter will always ask for administrator in order to run now
- meter will not work with traditional vpns, e.g. nordvpn, however exitlag and other ping reducers should still work
- removed the need to select an interface
- if the game is disconnecting randomly, try to close meter and see if it still is causing disconnects

#### BUG FIXES AND IMPROVEMENTS

- fixed buff tooltips showing incorrect percentages
- fixed paladin ark passive brand not grouping correctly
- fixed arcana evoke gem not showing correctly

### v1.18.0

#### NOTES

- thanks for the amazing work from molenzwiebel, poont, and several others, for the quick work on getting around the new damage data
  encryption.
- meter still does NOT touch the game files whatsoever. it does NOT read game memory, or tamper with easy anti-cheat.
- AGS can still detect meter like they always have. it always has been a gray area, however they have never issued any
  bans for it.
- due to the workarounds, meter MUST be opened before entering the raid
- due to the workarounds methods used, meter CANNOT be run in a vm anymore, raw socket also no longer works. meter should continue to
  work with vpns, and ping reducers like exitlag.

#### NEW FEATURES

- updated meter for aegir patch
- gem data added to skills on raid clear
- class spec data added to players on raid clear
- added a bunch of new tooltips to dps data

#### BUG FIXES AND IMPROVEMENTS

- fixed arcana knight card not showing
- fixed surge skill showing only skill id
- fixed artist t-skill buff showing in the wrong order
- improved local player detection

### v1.17.5

#### BUG FIXES AND IMPROVEMENTS

- fix some skills showing up wrong or missing

#### BUG FIXES AND IMPROVEMENTS

- fix skills with summons showing as different skill
- fix some icons missing

### v1.17.4

#### BUG FIXES AND IMPROVEMENTS

- attempt to fix buffs being calculated incorrectly
- attempt to fix some skills not being categorized correctly, e.g. scouter skills, re deathbalde
- fix some icons missing

### v1.17.0

#### NEW FEATURES

- updated meter for t4
- adjusted how buffs interact with hyper awakening
- added tracking for support hyper awakening technique buffs (internal for now)
- added uploading functionality (sidebar > upload), donations are appreciated to help keep servers running

#### BUG FIXES AND IMPROVEMENTS

- fix 10s dps graph not working
- added option to try to re-sync failed uploads, note: error logs can be found in the log file in the install directory

### v1.16.5

#### NEW FEATURES

- added button to show max damage for a cast in skill cast log

![max_cast](https://i.imgur.com/0ZNTm41.png)

- added button to launch lost ark within meter
- added option to auto launch lost ark when starting meter

![start_lost_ark](https://i.imgur.com/y4VNUth.png)

![auto_launch](https://i.imgur.com/WYxAfgt.png)

#### BUG FIXES AND IMPROVEMENTS

- fixed death count not showing in behemoth if everyone was alive at the end of the raid, but had deaths during the raid

### v1.16.4

#### NEW FEATURES

- updated app icon
- added quick setting toggles to logs

![toggles](https://i.imgur.com/roxHSBM.png)

- added death counts

![death_counts](https://i.imgur.com/dAwztWg.png)

#### BUG FIXES AND IMPROVEMENTS

- added esthers to split party view (oops i forgot them initially)
- fix searching by class

### v1.16.3

#### NEW FEATURES

- added party split damage option for logs only (settings > logs > split party damage: default ON)

#### BUG FIXES AND IMPROVEMENTS

- fix add in akkan raid counting as boss

### v1.16.2

#### BUG FIXES AND IMPROVEMENTS

- fixed artist sunsketch buff

### v1.16.1

#### NEW FEATURES

- update meter for behemoth patch

#### BUG FIXES AND IMPROVEMENTS

- fix meter crashes
- fix behemoth g2 not being labeled correctly

### v1.15.0

#### NEW FEATURES

- update meter for august patch

#### BUG FIXES AND IMPROVEMENTS

- attempt to fix meter crashing when windows language is set to certain locales
- attempt to fix stigma being counted as a brand when brand tripod not selected

### v1.14.2

#### BUG FIXES AND IMPROVEMENTS

- fix bard harp branding not tracking in certain gates

### v1.14.1

#### BUG FIXES AND IMPROVEMENTS

- fix "show latest encounter" not working when an encounter is already open
- fix certain skills that leave projectile behind not being tracked (e.g. reflux sorc frost call)

### v1.14.0

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
- due to log sizes increasing because of every single skill cast being saved, future encounters will have fields
  compressed, greatly reducing database sizes for the future

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

scrolling down to the bottom of the page allows you to view the cast in more detail. where you are able to hover over
columns for tooltips, and change the buff filtering to show different buffs applied during the damage tick. you can also
click the arrows to look at the next/previous cast of the skill. it will also show rdps data when that is back and
working again.

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
