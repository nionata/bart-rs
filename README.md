# bart-rs

A Rust workspace for the [BART Legacy API](https://api.bart.gov/docs/overview/index.aspx).

## Workspace

| Crate | Description |
|-------|-------------|
| `bart` | Async client library: typed models, JSON parsing, error handling |
| `bart-cli` | `bart` binary: stations, routes, real-time departure estimates |

## CLI

### Install

```sh
cargo install --path bart-cli
```

### Commands

#### `bart stations`

List all BART stations with their abbreviation, name, and location. Supports `--json`.

```sh
$ bart stations
```
```
12TH    12th St. Oakland City Center  (Oakland, CA)
16TH    16th St. Mission  (San Francisco, CA)
19TH    19th St. Oakland  (Oakland, CA)
24TH    24th St. Mission  (San Francisco, CA)
ANTC    Antioch  (Antioch, CA)
ASHB    Ashby  (Berkeley, CA)
BALB    Balboa Park  (San Francisco, CA)
BAYF    Bay Fair  (San Leandro, CA)
BERY    Berryessa/North San Jose  (San Jose, CA)
CAST    Castro Valley  (Castro Valley, CA)
CIVC    Civic Center/UN Plaza  (San Francisco, CA)
COLS    Coliseum  (Oakland, CA)
COLM    Colma  (Colma, CA)
CONC    Concord  (Concord, CA)
DALY    Daly City  (Daly City, CA)
DBRK    Downtown Berkeley  (Berkeley, CA)
DUBL    Dublin/Pleasanton  (Pleasanton, CA)
DELN    El Cerrito del Norte  (El Cerrito, CA)
PLZA    El Cerrito Plaza  (El Cerrito, CA)
EMBR    Embarcadero  (San Francisco, CA)
FRMT    Fremont  (Fremont, CA)
FTVL    Fruitvale  (Oakland, CA)
GLEN    Glen Park  (San Francisco, CA)
HAYW    Hayward  (Hayward, CA)
LAFY    Lafayette  (Lafayette, CA)
LAKE    Lake Merritt  (Oakland, CA)
MCAR    MacArthur  (Oakland, CA)
MLBR    Millbrae  (Millbrae, CA)
MLPT    Milpitas  (Milpitas, CA)
MONT    Montgomery St.  (San Francisco, CA)
NBRK    North Berkeley  (Berkeley, CA)
NCON    North Concord/Martinez  (Concord, CA)
OAKL    Oakland International Airport  (Oakland, CA)
ORIN    Orinda  (Orinda, CA)
PITT    Pittsburg/Bay Point  (Pittsburg, CA)
PCTR    Pittsburg Center  (Pittsburg, CA)
PHIL    Pleasant Hill/Contra Costa Centre  (Walnut Creek, CA)
POWL    Powell St.  (San Francisco, CA)
RICH    Richmond  (Richmond, CA)
ROCK    Rockridge  (Oakland, CA)
SBRN    San Bruno  (San Bruno, CA)
SFIA    San Francisco International Airport  (San Francisco Int'l Airport, CA)
SANL    San Leandro  (San Leandro, CA)
SHAY    South Hayward  (Hayward, CA)
SSAN    South San Francisco  (South San Francisco, CA)
UCTY    Union City  (Union City, CA)
WCRK    Walnut Creek  (Walnut Creek, CA)
WARM    Warm Springs/South Fremont  (Fremont, CA)
WDUB    West Dublin/Pleasanton  (Dublin, CA)
WOAK    West Oakland  (Oakland, CA)
```

#### `bart routes`

List all routes grouped by direction. Supports `-d north|south` to filter, `--json`, and `--no-icons`.

```sh
$ bart routes --no-icons
```
```
  North
    BLUE      Daly City to Dublin/Pleasanton
    GREEN     Daly City to Berryessa/North San Jose
    GREY      Oakland Int'l Airport OAK to Coliseum
    ORANGE    Berryessa/North San Jose to Richmond
    RED       Millbrae/SF Int'l Airport SFO to Richmond
    YELLOW    Millbrae/SF Int'l SFO to Antioch

  South
    BLUE      Dublin/Pleasanton to Daly City
    GREEN     Berryessa/North San Jose to Daly City
    GREY      Coliseum to Oakland Int'l Airport OAK
    ORANGE    Richmond to Berryessa/North San Jose
    RED       Richmond to SF Int'l Airport SFO/Millbrae
    YELLOW    Antioch to SF Int'l Airport SFO/Millbrae
```

#### `bart estimates <STATION>`

Real-time departure estimates for a station. Supports `-d north|south` to filter by direction, `--watch [N]` to refresh every N seconds (default 60), `--json`, and `--no-icons`.

```sh
$ bart estimates EMBR --no-icons
```
```
Embarcadero (EMBR)

  North
    BLUE      Dublin/Pleasanton           17 min, 39 min, 56 min
    RED       Richmond                    17 min, 33 min, 53 min
    YELLOW    Antioch                     20 min, 38 min, 58 min

  South
    YELLOW    SF Airport                   2 min, 14 min, 34 min
    BLUE      Daly City                    4 min, 18 min, 37 min
    RED       Millbrae                     6 min, 21 min, 41 min
```

## Library

See [`bart/src/lib.rs`](bart/src/lib.rs) for the full API reference.
