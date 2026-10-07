// Komponentene slik de rendres til teksturatlaset: ekte Fristil-markup,
// én oppføring per variant og tilstand. `zoom` skalerer hele komponenten
// (gateskiltene). Målene i motoren er 1 CSS-px = 1 cm.
export const SHEETS = [
  // Knappen ved porten.
  { id: "button:primary", html: '<button class="fs-button" type="button">Åpne porten</button>' },
  { id: "button:secondary", html: '<button class="fs-button" data-variant="secondary" type="button">Åpne porten</button>' },
  { id: "button:ghost", html: '<button class="fs-button" data-variant="ghost" type="button">Åpne porten</button>' },
  { id: "button:danger", html: '<button class="fs-button" data-variant="danger" type="button">Åpne porten</button>' },
  // Sendeknappen på skjemaet er også en .fs-button, med de samme variantene.
  ...[["primary", ""], ["secondary", ' data-variant="secondary"'], ["ghost", ' data-variant="ghost"'], ["danger", ' data-variant="danger"']].map(([v, a]) => ({
    id: `submit:${v}`,
    html: `<button class="fs-button"${a} type="button">Send søknad</button>`,
  })),
  { id: "button:search", html: '<button class="fs-button" type="button">Søk</button>' },
  { id: "button:upload", html: '<button class="fs-button" data-variant="secondary" type="button">Last opp vedlegg</button>' },
  // Badge: tavla og gateskiltene.
  ...["accent", "success", "warning", "danger", "neutral"].flatMap((c) =>
    [["Under behandling", "behandling"], ["Innvilget", "innvilget"], ["Mangler vedlegg", "mangler"], ["Avslått", "avslatt"], ["Arkivert", "arkivert"]].map(([t, k]) => ({
      id: `badge:${c}:${k}`,
      html: `<span class="fs-badge"${c === "accent" ? "" : ` data-color="${c}"`}>${t}</span>`,
    }))),
  // Gateskiltene: badgen i alle fem fargene, forstørret tre ganger.
  ...["Nordgata", "Sørgata", "Østre gate", "Vestre gate"].flatMap((n) =>
    ["accent", "success", "warning", "danger", "neutral"].map((c) => ({ id: `sign:${n}:${c}`, html: `<span class="fs-badge"${c === "accent" ? "" : ` data-color="${c}"`}>${n}</span>`, zoom: 3 }))),
  // Veiviserne ut av byen: lenker til stedene utenfor, med avstand, i
  // lenkas tre tilstander (samme varianter som link).
  ...[["Linelord", "12 km"], ["Streamlord", "7 km"], ["Cobble", "3 km"], ["Fristad", "1 km"]].flatMap(([n, d]) =>
    [["default", ""], ["visited", ""], ["disabled", ' aria-disabled="true"']].map(([v, a]) => ({
      id: `place:${n}:${v}`,
      html: `<a class="fs-link${v === "visited" ? " fs-link--visited" : ""}" href="#"${a}>${n}</a><span style="margin-left:12px;color:var(--fs-color-neutral-text-subtle)">${d}</span>`,
      zoom: 2,
    }))),
  // Tag.
  ...["Bostøtte", "Barnehage", "Arkivert", "Innvilget", "Klage"].flatMap((t) => [
    { id: `tag:default:${t}`, html: `<span class="fs-tag">${t}</span>` },
    { id: `tag:filled:${t}`, html: `<span class="fs-tag" data-variant="filled">${t}</span>` },
    { id: `tag:disabled:${t}`, html: `<span class="fs-tag" aria-disabled="true">${t}</span>` },
  ]),
  // Alert.
  ...[["neutral", ""], ["info", ' data-color="info"'], ["success", ' data-color="success"'], ["warning", ' data-color="warning"'], ["danger", ' data-color="danger"']].map(([c, a]) => ({
    id: `alert:${c}`,
    html: `<div class="fs-alert"${a} style="width:420px"><p class="fs-alert__title">Saken er registrert</p><p>Du får svar innen tre uker. Vi sender deg en e-post når den er behandlet.</p></div>`,
  })),
  // Breadcrumbs.
  { id: "breadcrumbs", html: '<nav aria-label="Du er her"><ol class="fs-breadcrumbs"><li><a href="#">Forsiden</a></li><li><a href="#">Torget</a></li><li><a href="#" aria-current="page">Porten</a></li></ol></nav>' },
  // Link.
  ...[["default", ""], ["visited", ' data-visited'], ["disabled", ' aria-disabled="true"']].flatMap(([v, a]) =>
    ["Mine søknader", "Torget", "Dokumentasjonen"].map((t) => ({ id: `link:${v}:${t}`, html: `<a class="fs-link${v === "visited" ? " fs-link--visited" : ""}" href="#"${a}>${t}</a>`, zoom: 1.25 }))),
  { id: "link:home", html: '<a class="fs-link" href="#">Torget</a>', zoom: 1.25 },
  // Divider.
  { id: "divider:default", html: '<hr class="fs-divider" style="width:300px" />' },
  { id: "divider:strong", html: '<hr class="fs-divider" data-variant="strong" style="width:300px" />' },
  { id: "divider:subtle", html: '<hr class="fs-divider" data-variant="subtle" style="width:300px" />' },
  // Heading.
  ...[["default", ""], ["mega", ' data-size="mega"'], ["xl", ' data-size="xl"'], ["m", ' data-size="m"'], ["s", ' data-size="s"'], ["xs", ' data-size="xs"']].map(([v, a]) => ({ id: `heading:${v}`, html: `<h2 class="fs-heading"${a}>Vedlegg til søknaden</h2>` })),
  // Label, help-text, error-text.
  ...[["default", ""], ["required", ' data-required="symbol"'], ["required-text", ' data-required="text"'], ["optional", ' data-optional=""'], ["disabled", ' aria-disabled="true"']].map(([v, a]) => ({ id: `label:${v}`, html: `<label class="fs-label"${a}>Fødselsnummer</label>` })),
  { id: "help-text:default", html: '<p class="fs-help-text">Du finner nummeret på skattekortet.</p>' },
  { id: "help-text:strong", html: '<p class="fs-help-text" data-variant="strong">Nummeret må stemme med folkeregisteret.</p>' },
  { id: "help-text:success", html: '<p class="fs-help-text" data-variant="success">Nummeret er kontrollert.</p>' },
  { id: "help-text:warning", html: '<p class="fs-help-text" data-variant="warning">Nummeret ser ut som et D-nummer.</p>' },
  { id: "error-text:default", html: '<p class="fs-error-text">Skriv fødselsnummeret med 11 siffer, uten mellomrom.</p>' },
  { id: "error-text:warning", html: '<p class="fs-error-text" data-variant="warning">Nummeret finnes ikke i folkeregisteret. Sjekk at det stemmer.</p>' },
  // Fieldset-legend.
  ...[["default", ""], ["required", ' data-required="symbol"'], ["invalid", ""], ["success", ""], ["disabled", ' aria-disabled="true"']].map(([v, a]) => ({
    id: `fieldset:${v}`,
    html: `<fieldset class="fs-fieldset"${v === "invalid" ? ' data-state="invalid"' : v === "success" ? ' data-state="success"' : ""}><legend class="fs-legend"${a}>Vedlegg</legend></fieldset>`,
  })),
  // Progress.
  ...["accent", "success", "warning", "danger"].map((c) => ({ id: `progress:${c}`, html: `<progress class="fs-progress" value="100" max="100"${c === "accent" ? "" : ` data-color="${c}"`} style="width:300px"></progress>` })),
  { id: "progress:track", html: '<progress class="fs-progress" value="0" max="100" style="width:300px"></progress>' },
  // Spinner.
  { id: "spinner:small", html: '<span class="fs-spinner" data-size="small" aria-hidden="true"></span>' },
  { id: "spinner:medium", html: '<span class="fs-spinner" aria-hidden="true"></span>' },
  { id: "spinner:large", html: '<span class="fs-spinner" data-size="large" aria-hidden="true"></span>' },
  // Switch.
  { id: "switch:on", html: '<input class="fs-switch" type="checkbox" role="switch" checked />' },
  { id: "switch:off", html: '<input class="fs-switch" type="checkbox" role="switch" />' },
  { id: "switch:disabled", html: '<input class="fs-switch" type="checkbox" role="switch" disabled />' },
  { id: "switch:label", html: '<label class="fs-label">Gatelysene</label>' },
  // Checkbox og radio.
  ...[["default", ""], ["invalid", ' data-state="invalid"'], ["disabled", " disabled"]].flatMap(([v, a]) => [
    { id: `checkbox:${v}:off`, html: `<input class="fs-checkbox" type="checkbox"${a} />` },
    { id: `checkbox:${v}:on`, html: `<input class="fs-checkbox" type="checkbox" checked${a} />` },
  ]),
  { id: "checkbox:indeterminate", html: '<input class="fs-checkbox" type="checkbox" data-indeterminate />', indeterminate: true },
  ...[["default", ""], ["invalid", ' data-state="invalid"'], ["disabled", " disabled"]].flatMap(([v, a]) => [
    { id: `radio:${v}:off`, html: `<input class="fs-radio" type="radio"${a} />` },
    { id: `radio:${v}:on`, html: `<input class="fs-radio" type="radio" checked${a} />` },
  ]),
  // Input, select, textarea, search, file-upload: tomme felt per tilstand.
  ...[["default", ""], ["success", ' data-state="success"'], ["invalid", ' data-state="invalid"'], ["disabled", " disabled"]].flatMap(([v, a]) => [
    { id: `input:${v}`, html: `<input class="fs-input" type="email"${a} style="width:360px" />` },
    { id: `textarea:${v}`, html: `<textarea class="fs-textarea" rows="3"${a} style="width:360px;height:80px"></textarea>` },
    { id: `search:${v}`, html: `<input class="fs-input fs-search" type="search"${a} style="width:250px" />` },
    { id: `file-upload:${v}`, html: `<input class="fs-file-upload" type="file"${a} style="width:360px" />` },
  ]),
  ...[["default", ""], ["invalid", ' data-state="invalid"'], ["disabled", " disabled"]].map(([v, a]) => ({
    id: `select:${v}`,
    html: `<select class="fs-select"${a} style="width:360px"><option>Velg fylke</option></select>`,
  })),
  { id: "select:list", html: '<ul class="fs-select-list" style="width:360px;list-style:none;margin:0;padding:4px 0;background:var(--fs-color-neutral-canvas);border:1px solid var(--fs-color-neutral-border);border-radius:4px"><li style="padding:8px 12px">Velg fylke</li><li style="padding:8px 12px">Innlandet</li><li style="padding:8px 12px">Vestland</li><li style="padding:8px 12px">Troms</li></ul>' },
  // Card.
  ...[["default", ""], ["filled", ' data-variant="filled"'], ["interactive", " data-interactive"]].map(([v, a]) => ({
    id: `card:${v}`,
    html: `<div class="fs-card"${a} style="width:360px"><h3 class="fs-card__title">Søknad om bostøtte</h3><p>Sendt 4. mars. Saksnummer 2026-0481.</p><span class="fs-badge" data-color="success">Innvilget</span></div>`,
  })),
  // Accordion.
  ...[["default", ""], ["plain", ' data-variant="plain"']].flatMap(([v, a]) => [
    { id: `accordion:${v}:closed`, html: `<details class="fs-accordion"${a} style="width:360px"><summary>Hvem kan søke om bostøtte?</summary><div class="fs-accordion__content"><p>Alle som bor og er folkeregistrert i boligen kan søke.</p></div></details>` },
    { id: `accordion:${v}:open`, html: `<details class="fs-accordion"${a} open style="width:360px"><summary>Hvem kan søke om bostøtte?</summary><div class="fs-accordion__content"><p>Alle som bor og er folkeregistrert i boligen kan søke. Du må være over 18 år, eller ha egne barn boende hos deg.</p></div></details>` },
    // Det andre panelet i skapet, lukket.
    { id: `accordion2:${v}:closed`, html: `<details class="fs-accordion"${a} style="width:360px"><summary>Hvor lang er behandlingstiden?</summary><div class="fs-accordion__content"><p>Vi behandler søknaden innen tre uker.</p></div></details>` },
  ]),
  // Table.
  ...[["default", ""], ["striped", ' data-variant="striped"'], ["hoverable", " data-hoverable"]].map(([v, a]) => ({
    id: `table:${v}`,
    html: `<table class="fs-table"${a} style="width:420px"><caption>Rutetabell</caption><thead><tr><th scope="col">Linje</th><th scope="col">Til</th><th scope="col" data-align="end">Avgang</th></tr></thead><tbody><tr><th scope="row">1</th><td>Tårnene</td><td data-align="end">08:10</td></tr><tr${v === "hoverable" ? ' data-hover' : ""}><th scope="row">2</th><td>Tokenparken</td><td data-align="end">08:25</td></tr><tr><th scope="row">3</th><td>Verkstedet</td><td data-align="end">08:40</td></tr></tbody></table>`,
  })),
  // Tabs.
  ...[0, 1, 2].map((k) => ({
    id: `tabs:${k}`,
    html: `<div class="fs-tabs" role="tablist">${["Søknaden", "Vedlegg", "Vedtak"].map((t, i) => `<button class="fs-tabs__tab" role="tab" aria-selected="${i === k}" type="button">${t}</button>`).join("")}</div>`,
  })),
  // Avatar.
  { id: "avatar:small", html: '<span class="fs-avatar" data-size="small">ON</span>' },
  { id: "avatar:medium", html: '<span class="fs-avatar">ON</span>' },
  { id: "avatar:large", html: '<span class="fs-avatar" data-size="large">ON</span>' },
  { id: "avatar:circle", html: '<span class="fs-avatar">KB</span>' },
  { id: "avatar:square", html: '<span class="fs-avatar" data-variant="square">KB</span>' },
  { id: "avatar:stack", html: '<ul class="fs-avatar-stack"><li><span class="fs-avatar" data-size="small">ON</span></li><li><span class="fs-avatar" data-size="small">KB</span></li><li><span class="fs-avatar" data-size="small">MH</span></li></ul>' },
  // Tooltip.
  { id: "tooltip:bubble", html: '<span class="fs-tooltip__bubble" role="tooltip" style="position:static;display:inline-block">Saken flyttes til arkivet og kan hentes fram igjen</span>' },
  { id: "tooltip:button", html: '<button class="fs-button" data-variant="ghost" type="button">Arkiver</button>' },
  // Dialog.
  { id: "dialog", html: '<dialog class="fs-dialog" open style="position:static;margin:0;width:420px"><h2 class="fs-dialog__title">Slette søknaden?</h2><div class="fs-dialog__body"><p>Søknaden og vedleggene blir borte. Dette kan ikke angres.</p></div><div class="fs-dialog__footer"><button class="fs-button" data-variant="secondary" type="button">Avbryt</button><button class="fs-button" data-variant="danger" type="button">Slett søknaden</button></div></dialog>' },
  // Skeleton.
  { id: "skeleton:block", html: '<div class="fs-skeleton" style="width:200px;height:120px"></div>' },
  { id: "skeleton:text", html: '<div class="fs-skeleton" data-variant="text" style="width:200px"></div>' },
  { id: "skeleton:circle", html: '<div class="fs-skeleton" data-variant="circle"></div>' },
  // Connection status.
  { id: "connection-status:online", html: '<div class="fs-connection-status" data-state="online" style="display:block;position:static">Tilkoblet</div>' },
  { id: "connection-status:offline", html: '<div class="fs-connection-status" data-state="offline" style="display:block;position:static">Ingen forbindelse</div>' },
];
