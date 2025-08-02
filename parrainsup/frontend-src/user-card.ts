import "./user-card.scss";
import { LitElement, html } from "lit-element";
import { property, customElement } from "lit-element/decorators.js";
import { MainUser } from "./api";
@customElement("user-card")
export class MainUserCard extends LitElement {
    data: MainUser;

    constructor(data: MainUser) {
        super();
        this.data = data;
    }
    createRenderRoot() {
        return this; // Render template in light DOM instead of shadow dom
    }

    updateData(newData: MainUser) {
        this.data = newData;
        this.requestUpdate();
    }

    render() {
        this.style.borderLeftColor = this.data.couleur;
        return html`
            <div class="container-first-row">
                <p class="display_name" title="Vrai Nom">
                    ${this.data.display_name}
                </p>
                ${!this.data.pronouns
                    ? html``
                    : html`<p class="pronouns" title="Pronouns">
                          (${this.data.pronouns})
                      </p>`}
            </div>
            <div class="container-second-row">
                ${!this.data.surnom
                    ? html``
                    : html`<p class="surnom" title="Surnom">
                          ≈ ${this.data.surnom}
                      </p>`}
                ${!this.data.discord_username
                    ? html``
                    : html`<p
                          class="discord_username"
                          title="Nom d'utilisateur Discord">
                          <iconify-icon
                              icon="mdi:discord"
                              width="20"
                              height="20"></iconify-icon>
                          ${this.data.discord_username}
                      </p>`}
            </div>
            <div class="sep"></div>
            <div
                class="container-third-row${this.data.voeu && this.data.origine
                    ? " both"
                    : ""}">
                ${!!this.data.origine
                    ? html`
                          <label class="label-origine">Lycée d'origine:</label>
                          <p class="origine" title="Lycée d'origine">
                              ${this.data.origine}
                          </p>
                      `
                    : html``}
                ${!!this.data.voeu
                    ? html`
                          <label class="label-voeu">École visée:</label>
                          <p class="voeu" title="École visée">
                              ${this.data.voeu}
                          </p>
                      `
                    : html``}
                ${!!this.data.voeu && !!this.data.origine
                    ? html` <label class="label-arrow"
                          ><iconify-icon
                              icon="mdi:arrow"
                              width="24"
                              height="24"></iconify-icon
                      ></label>`
                    : html``}
            </div>
            <div style="height: 10px"></div>
            <div class="container-questions">
                ${!this.data.fun_fact
                    ? html``
                    : html` <label>Fun Fact:</label>
                          <!-- prettier-ignore -->
                          <p class="fun_fact" title="Fun Fact">${this.data
                              .fun_fact}</p>`}
                ${!this.data.conseil
                    ? html``
                    : html` <label>Conseil pour les bizuths:</label>
                          <!-- prettier-ignore -->
                          <p class="conseil" title="Conseil pour les bizuths">${this
                              .data.conseil}</p>`}
                ${!this.data.algebre_or_analyse
                    ? html``
                    : html` <label>Algèbre ou analyse?</label>
                          <p class="algebre_or_analyse">
                              ${this.data.algebre_or_analyse}
                          </p>`}
                ${!this.data.c_or_ocaml
                    ? html``
                    : html` <label>C ou Ocaml:</label>
                          <p class="c_or_ocaml" title="Langage préféré">
                              ${this.data.c_or_ocaml}
                          </p>`}
            </div>
        `;
    }
}
