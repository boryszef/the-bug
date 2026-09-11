### Polish strings — a first draft for a Polish speaker to review and
### correct (see docs/i18n-plan.md). Grammatical case is only resolved where
### `src/i18n/mod.rs` explicitly asks for it (terrain locative in
### `event-found`, item genitive plural in the "not enough X" events, item
### accusative as the direct object of "Tworzysz"/"zrobić" in the crafting
### events); other interpolated nouns fall back to the nominative form.

## Event log lines.

event-awoke = Budzisz się i postanawiasz zbadać okolicę.
event-found = Znajdujesz { $item } { $terrain }.
event-found-poi = Znajdujesz { $item } { $poi }.
event-quest-accepted = Zadanie przyjęte: { $quest }
event-quest-completed = Zadanie ukończone: { $quest }!
event-crafted = Tworzysz { $output }.
event-experiment-failed = Eksperyment: { $items } -> klapa
event-experiment-missing-tool = Eksperyment: { $items } -> brakuje narzędzia
event-experimented = Eksperyment: { $items } -> { $output }{ $newly_learned ->
    [yes] { " (nowy przepis!)" }
   *[no] {""}
}
event-disassembled = Rozbierasz { $item }, odzyskując { $recovered }.
event-hunted = Polowanie: { $items }.
event-hunt-missed = Polowanie: zwierzyna uciekła.
event-equipment-full = Nie masz już miejsca na { $item }.
event-dropped = Wyrzucasz { $item }.
event-leveled-up = Awansujesz! Jesteś teraz na poziomie { $level }.

## Item names (nominative), with a `.genitive` attribute (singular), a
## `.genitive-plural` attribute — the latter is what the "not enough X"
## events use, since Polish wants the plural genitive for a shortage of a
## countable noun ("za mało kamieni", not "za mało kamień") — and an
## `.accusative` attribute (singular) for when the item is the direct
## object of a crafting verb ("Tworzysz Strzałę", not "Tworzysz Strzała").
## Where the nominative and accusative coincide (masculine/neuter nouns, and
## consonant-stem feminine ones like "gałąź"), the `.accusative` just repeats
## the name.

item-branch = Gałąź
    .genitive = gałęzi
    .genitive-plural = gałęzi
    .accusative = Gałąź
item-stone = Kamień
    .genitive = kamienia
    .genitive-plural = kamieni
    .accusative = Kamień
item-vine = Pnącze
    .genitive = pnącza
    .genitive-plural = pnączy
    .accusative = Pnącze
item-cord = Sznurek
    .genitive = sznurka
    .genitive-plural = sznurków
    .accusative = Sznurek
item-stone-axe = Kamienny Topór
    .genitive = kamiennego topora
    .genitive-plural = kamiennych toporów
    .accusative = Kamienny Topór
item-arrow = Strzała
    .genitive = strzały
    .genitive-plural = strzał
    .accusative = Strzałę
item-wooden-bow = Drewniany Łuk
    .genitive = drewnianego łuku
    .genitive-plural = drewnianych łuków
    .accusative = Drewniany Łuk
item-plastic-bottle = Plastikowa Butelka
    .genitive = plastikowej butelki
    .genitive-plural = plastikowych butelek
    .accusative = Plastikową Butelkę
item-copper-wire = Miedziany Drut
    .genitive = miedzianego drutu
    .genitive-plural = miedzianych drutów
    .accusative = Miedziany Drut
item-coil = Cewka
    .genitive = cewki
    .genitive-plural = cewek
    .accusative = Cewkę
item-pole = Drążek
    .genitive = drążka
    .genitive-plural = drążków
    .accusative = Drążek
item-microcontroller = Mikrokontroler
    .genitive = mikrokontrolera
    .genitive-plural = mikrokontrolerów
    .accusative = Mikrokontroler
item-speaker = Głośnik
    .genitive = głośnika
    .genitive-plural = głośników
    .accusative = Głośnik
item-metal-detector = Wykrywacz Metalu
    .genitive = wykrywacza metalu
    .genitive-plural = wykrywaczy metalu
    .accusative = Wykrywacz Metalu
item-battery = Bateria
    .genitive = baterii
    .genitive-plural = baterii
    .accusative = Baterię
item-solar-panel = Panel Słoneczny
    .genitive = panelu słonecznego
    .genitive-plural = paneli słonecznych
    .accusative = Panel Słoneczny
item-solar-charger = Ładowarka Słoneczna
    .genitive = ładowarki słonecznej
    .genitive-plural = ładowarek słonecznych
    .accusative = Ładowarkę Słoneczną
item-circuit-board = Płytka Drukowana
    .genitive = płytki drukowanej
    .genitive-plural = płytek drukowanych
    .accusative = Płytkę Drukowaną
item-umbrella = Parasol
    .genitive = parasola
    .genitive-plural = parasoli
    .accusative = Parasol
item-fabric = Tkanina
    .genitive = tkaniny
    .genitive-plural = tkanin
    .accusative = Tkaninę
item-electronic-toy = Elektroniczna Zabawka
    .genitive = elektronicznej zabawki
    .genitive-plural = elektronicznych zabawek
    .accusative = elektroniczną zabawkę
item-rusty-metal = Zardzewiały Metal
    .genitive = zardzewiałego metalu
    .genitive-plural = zardzewiałego metalu
    .accusative = zardzewniały metal
item-metal-knife = Metalowy Nóż
    .genitive = metalowego noża
    .genitive-plural = metalowych noży
    .accusative = metalowy nóż
item-electric-motor = Silnik Elektryczny
    .genitive = silnika elektrycznego
    .genitive-plural = silników elektrycznych
    .accusative = silnik elektryczny
item-steel-bolt = Stalowy Nit
    .genitive = stalowego nitu
    .genitive-plural = stalowych nitów
    .accusative = stalowy nit
item-meat = Mięso
    .genitive = mięsa
    .genitive-plural = mięsa
    .accusative = Mięso
item-bone = Kość
    .genitive = kości
    .genitive-plural = kości
    .accusative = Kość
item-hide = Skóra
    .genitive = skóry
    .genitive-plural = skór
    .accusative = Skórę
item-fur = Futro
    .genitive = futra
    .genitive-plural = futer
    .accusative = Futro
item-satchel = Sakwa
    .genitive = sakwy
    .genitive-plural = sakiew
    .accusative = sakwę
item-bone-needle = Kościana Igła
    .genitive = kościanej igły
    .genitive-plural = kościanych igieł
    .accusative = kościaną igłę

## Terrain names (nominative), with a `.locative` attribute (the full
## prepositional phrase, e.g. "w lesie" for "in the Forest") used by
## event-found instead of a separate glue word.

terrain-meadow = Łąka
    .locative = na łące
terrain-forest = Las
    .locative = w lesie
terrain-deadland = Pustkowie
    .locative = na pustkowiu

## Point-of-interest names (nominative), with the same `.locative` attribute.

poi-cave = Jaskinia
    .locative = w jaskini
poi-ruins = Ruiny
    .locative = w ruinach
poi-village = Osada
    .locative = w osadzie

## Quest name/description.

quest-craft-axe-name = Kłopoty na wschodzie
quest-craft-axe-description = Coś znów szykuje się na wschodzie. Od jakiegoś czasu było spokojnie, ale doświadczenie pokazuje, że kłopoty zawsze nadchodzą z tamtej strony. Starzy ludzie mają na to swoje historie — że pierwsi osadnicy musieli walczyć z samą ziemią, żeby przetrwać, że nawet rośliny jakby chciały się ich pozbyć. Niektórzy twierdzą, że dawny las umiał myśleć, planować, czekać. Bzdury, mówi większość, kręcąc głowami — rośliny nie intrygują. Ale i tak nie możesz się powstrzymać od zastanawiania się. Cokolwiek się szykuje, lepiej się przygotuj! Idź do lasu, na łąkę i do jaskini, zbierz materiały, a potem poeksperymentuj z nimi i naucz się jak zrobić kamienną siekierę.
quest-craft-cord-name = Po nitce do kłębka
quest-craft-cord-description = Miedziany drut się przyda, ale połowa tego, co zechcesz zbudować, wymaga czegoś prostszego — kawałka porządnego sznurka. W osadzie nikt już nie zawraca sobie głowy elektrycznością; dziś liczy się nić, węzeł, pole i kuźnia. Jest też taka stara nieufność wobec urządzeń w ruinach — coś w rodzaju niechęci do rzeczy, które kiedyś myślały za ludzi, choć nikt już nie pamięta, dlaczego właściwie. A jednak trudno nie zauważyć tych cienkich, błyszczących płyt porozrzucanych wśród gruzu, gładkich jak spokojna woda pod warstwą brudu. Winorośl łatwo znaleźć — spróbuj połączyć kilka kawałków i zobacz, co się uda. Czasem naprawdę, jak to mówią, po nitce do kłębka.
quest-disassemble-umbrella-name = Skarby ze złomu
quest-disassemble-umbrella-description = Stara, zepsuta do cna parasolka wciąż ma w sobie dobre części — materiał, metalowy drążek, może coś więcej, jeśli przyjrzysz się bliżej. Ostatnio rozbierasz na części wszystko, co oddają ruiny, żeby zobaczyć, jak to zrobiono i co warto zachować. Stara technologia często chowa w sobie coś użytecznego, jeśli tylko zechcesz porządnie ją rozłożyć na części, a nie zostawić całą i bezużyteczną. Gdzieś tam na pewno leży jedna taka parasolka, czeka, żeby ją znaleźć.
quest-explore-ruins-name = Cywilizacja Cyfrowa
quest-explore-ruins-description = Przejezdny podróżny wspomniał o pobliskich ruinach, podobno usianych starymi przedmiotami. Podobno na tych równinach znajdowało się niegdyś kilka wiosek - pozostałości wielkiej cywilizacji sprzed 500 lat — "Cywilizacji Cyfrowej", jak nazywają ją odkrywcy. Powinieneś zobaczyć to na własne oczy.
quest-old-civilization-name = Co kryją ruiny
quest-old-civilization-description = Ruiny sięgają głębiej, niż się wydawało — fundamenty, mury, całe pomieszczenia na wpół pochłonięte przez ziemię. Cywilizacja Cyfrowa, jak mówią mieszkańcy osady, żyła z elektryczności i urządzeń, które trochę myślały za człowieka — skrzynek, płyt i ekranów w każdym kształcie, wplecionych we wszystko, co budowali. Większość z tego już nie działa, skorodowana do tego stopnia, że nie da się odróżnić jednej maszyny od drugiej. Pogrzeb się w ziemi i zobacz, co jeszcze skrywa. Miedziany drut byłby dobrym początkiem — każdy majsterkowicz w osadzie się ucieszy.
quest-stock-up-name = Zapasy na ciężkie czasy
quest-stock-up-description = Ktoś doświadczony w osadzie wciąż zerka na niebo. Idą chłody, mówi, a zapasy nie wystarczą dla wszystkich — mięso, skóra, kości, futro, co tylko zdołasz przynieść, zanim nadejdą mrozy. Mówi też, żebyś miał oczy szeroko otwarte: łąka i las nie są już całkiem takie, jak opowiadają stare historie, choć sama nie umie powiedzieć, co się zmieniło. Weź łuk. Pięć porządnych polowań to dobry początek.

## Panel titles.

panel-map-title = Mapa
panel-player-title = Gracz
panel-events-title = Zdarzenia
panel-craft-title = Wytwórz
panel-recipes-title = Przepisy
panel-experiment-title = Eksperyment
panel-available-title = Dostępne
panel-selected-title = Wybrane
panel-disassemble-title = Rozmontuj
panel-items-title = Przedmioty
panel-quests-title = Zadania
panel-active-title = Aktywne
panel-completed-title = Ukończone

## Player panel.

player-level = Poziom: { $level }
player-xp = XP: { $xp }
player-recipes = Przepisy: { $known }/{ $total }
player-equipment = Wyposażenie: { $carried }/{ $capacity }
player-quests = Zadania: { $completed }/{ $total } — { $active }
player-quest-none = (brak)

## Village-gated actions: the gui disables the Craft/Experiment/Disassemble
## buttons away from the Village and shows this hint instead — no event is
## logged (a refusal here is the player's own doing, not something the game
## did, so it isn't event-log material; see docs/village-crafting.md).

village-required-hint = Musisz być w osadzie, aby to zrobić.

## Items panel (the Equipment/Inventory split; see
## docs/equipment-and-storage.md). "panel-items-title" above doubles as this
## tab's heading.

items-equipment-title = Ekwipunek: { $count }/{ $capacity }
items-storage-title = Magazyn
items-equipment-empty = Twój ekwipunek jest pusty.
items-storage-empty = Magazyn jest pusty.

## Experiment panel.

experiment-promising = Wygląda dobrze!

## Craft panel.

craft-empty = Nie odkryłeś jeszcze żadnych przepisów.

## Disassemble panel.

disassemble-empty = Nie masz nic, co dałoby się rozłożyć.

## Quests panel.

quests-active-none = Nie przyjęto żadnego zadania.
quests-active-blocked = Ukończ aktywne zadanie, aby przyjąć kolejne.
quests-available-empty = Brak dostępnych zadań.
quests-completed-empty = Nie ukończono jeszcze żadnego zadania.

## Sterowanie (pasek narzędzi, przyciski paneli, pasek podpowiedzi w src/gui/mod.rs).

action-quit = Wyjście (q)
action-experiment = Eksperymentuj
action-accept = Przyjmij
action-search = Szukaj (s)
action-hunt = Poluj (h)
action-theme-light = Jasny
action-theme-dark = Ciemny
font-size-label = Czcionka:
font-size-small = Mała
font-size-medium = Średnia
font-size-large = Duża
font-size-extra-large = Bardzo duża
gui-hint = [ ] zmiana kart   q wyjście
gui-hint-map = przeciągnij, aby przesunąć   przewiń, aby przybliżyć
