export const encounterMap: { [key: string]: { [key: string]: Array<string> } } = {
  Valtan: {
    "Valtan G1": ["Dark Mountain Predator", "Destroyer Lucas", "Leader Lugaru"],
    "Valtan G2": ["Demon Beast Commander Valtan", "Ravaged Tyrant of Beasts"]
  },
  Vykas: {
    "Vykas G1": ["Incubus Morphe", "Nightmarish Morphe"],
    "Vykas G2": ["Covetous Devourer Vykas"],
    "Vykas G3": ["Covetous Legion Commander Vykas"]
  },
  Clown: {
    "Clown G1": ["Saydon"],
    "Clown G2": ["Kakul"],
    "Clown G3": ["Kakul-Saydon", "Encore-Desiring Kakul-Saydon"]
  },
  Brelshaza: {
    "Brelshaza G1": ["Gehenna Helkasirs"],
    "Brelshaza G2": ["Prokel", "Prokel's Spiritual Echo", "Ashtarot"],
    "Brelshaza G3": ["Primordial Nightmare"],
    "Brelshaza G4": ["Phantom Legion Commander Brelshaza"],
    "Brelshaza G5": [
      "Brelshaza, Monarch of Nightmares",
      "Imagined Primordial Nightmare",
      "Pseudospace Primordial Nightmare"
    ],
    "Brelshaza G6": ["Phantom Legion Commander Brelshaza"]
  },
  Kayangel: {
    "Kayangel G1": ["Tienis"],
    "Kayangel G2": ["Prunya"],
    "Kayangel G3": ["Lauriel"]
  },
  Akkan: {
    "Akkan G1": ["Griefbringer Maurug", "Evolved Maurug"],
    "Akkan G2": ["Lord of Degradation Akkan"],
    "Akkan G3": ["Plague Legion Commander Akkan", "Lord of Kartheon Akkan"]
  },
  "Ivory Tower": {
    "Ivory Tower G1": ["Kaltaya, the Blooming Chaos"],
    "Ivory Tower G2": ["Rakathus, the Lurking Arrogance"],
    "Ivory Tower G3": ["Firehorn, Trampler of Earth"],
    "Ivory Tower G4": [
      "Lazaram, the Trailblazer",
      "Subordinated Vertus",
      "Subordinated Calventus",
      "Subordinated Legoros",
      "Brand of Subordination"
    ]
  },
  Thaemine: {
    "Thaemine G1": ["Killineza the Dark Worshipper"],
    "Thaemine G2": ["Valinak, Knight of Darkness", "Valinak, Taboo Usurper", "Valinak, Herald of the End"],
    "Thaemine G3": ["Thaemine the Lightqueller", "Dark Greatsword"],
    "Thaemine G4": ["Darkness Legion Commander Thaemine", "Thaemine Prokel", "Thaemine, Conqueror of Stars"]
  },
  Echidna: {
    "Echidna G1": ["Red Doom Narkiel", "Agris"],
    "Echidna G2": [
      "Echidna",
      "Covetous Master Echidna",
      "Desire in Full Bloom, Echidna",
      "Alcaone, the Twisted Venom",
      "Agris, the Devouring Bog"
    ]
  },
  Behemoth: {
    "Behemoth G1": [
      "Behemoth, the Storm Commander",
      "Despicable Skolakia",
      "Untrue Crimson Yoho",
      "Ruthless Lakadroff",
      "Vicious Argeos"
    ],
    "Behemoth G2": ["Behemoth, Cruel Storm Slayer"]
  },
  Aegir: {
    "Aegir G1": ["Akkan, Lord of Death", "Abyss Monarch Aegir"],
    "Aegir G2": ["Aegir, the Oppressor", "Pulsating Giant's Heart"]
  },
  "Act 2: Brelshaza": {
    "Act 2: Brelshaza G1": ["Narok the Butcher"],
    "Act 2: Brelshaza G2": ["Phantom Legion Commander Brelshaza", "Phantom Manifester Brelshaza"]
  },
  "Act 3: Mordum": {
    "Act 3: Mordum G1": ["Thaemine, Master of Darkness", "Infernas"],
    "Act 3: Mordum G2": ["Blossoming Fear, Naitreya"],
    "Act 3: Mordum G3": ["Mordum, the Abyssal Punisher", "Mordum's Hammer", "Flash of Punishment"]
  }
};

export const bossHpMap: Record<string, number> = {
  "Dark Mountain Predator": 50,
  "Destroyer Lucas": 50,
  "Leader Lugaru": 50,
  "Demon Beast Commander Valtan": 160,
  "Ravaged Tyrant of Beasts": 40,
  "Incubus Morphe": 60,
  "Nightmarish Morphe": 60,
  "Covetous Devourer Vykas": 160,
  "Covetous Legion Commander Vykas": 180,
  Saydon: 160,
  Kakul: 140,
  "Kakul-Saydon": 180,
  "Encore-Desiring Kakul-Saydon": 77,
  "Gehenna Helkasirs": 120,
  Ashtarot: 170,
  "Primordial Nightmare": 190,
  "Brelshaza, Monarch of Nightmares": 200,
  "Imagined Primordial Nightmare": 20,
  "Pseudospace Primordial Nightmare": 20,
  "Phantom Legion Commander Brelshaza": 250,
  "Griefbringer Maurug": 150,
  "Evolved Maurug": 30,
  "Lord of Degradation Akkan": 190,
  "Plague Legion Commander Akkan": 220,
  "Lord of Kartheon Akkan": 300,
  Tienis: 110,
  "Celestial Sentinel": 60,
  Prunya: 90,
  Lauriel: 200,
  "Kaltaya, the Blooming Chaos": 120,
  "Rakathus, the Lurking Arrogance": 160,
  "Firehorn, Trampler of Earth": 160,
  "Lazaram, the Trailblazer": 200,
  "Killineza the Dark Worshipper": 180,
  "Valinak, Knight of Darkness": 180,
  "Valinak, Taboo Usurper": 180,
  "Valinak, Herald of the End": 180,
  "Thaemine the Lightqueller": 300,
  "Dark Greatsword": 40,
  "Darkness Legion Commander Thaemine": 350,
  "Thaemine Prokel": 35,
  "Thaemine, Conqueror of Stars": 350,
  "Red Doom Narkiel": 180,
  Agris: 100,
  Echidna: 285,
  "Covetous Master Echidna": 285,
  "Alcaone, the Twisted Venom": 86,
  "Agris, the Devouring Bog": 103,
  "Behemoth, the Storm Commander": 500,
  "Behemoth, Cruel Storm Slayer": 705,
  "Akkan, Lord of Death": 220,
  "Aegir, the Oppressor": 300,
  "Narok the Butcher": 300,
  "Phantom Manifester Brelshaza": 420,
  "Thaemine, Master of Darkness": 300,
  Infernas: 300,
  "Blossoming Fear, Naitreya": 300,
  "Mordum, the Abyssal Punisher": 500,
  "Flash of Punishment": 350
};

export const difficultyMap: Array<string> = [
  "Normal",
  "Hard",
  "Inferno",
  "Challenge",
  "Solo",
  "Trial",
  "Extreme",
  "The First"
];

export const raidGates: Record<string, string> = Object.fromEntries(
  Object.values(encounterMap)
    .flatMap((raid) => Object.entries(raid))
    .flatMap(([gate, bosses]) => bosses.map((boss) => [boss, gate]))
);

export const bossList = [
  // guardian raids
  "Drextalas",
  "Skolakia",
  "Argeos",
  "Veskal",
  "Gargadeth",
  "Sonavel",
  "Hanumatan",
  "Kungelanium",
  "Deskaluda"
];

export const bossHpBarColors = ["#D16F23", "#9F3930", "#582469", "#2B3A63", "#246977", "#798816", "#E7B826"];
