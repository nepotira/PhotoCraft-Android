# 🤖 Diretrizes de Persistência e Manutenção da IA - PhotoCraft Android

> **DOCUMENTO MANDATÓRIO PARA A IA ASSISTENTE:**
> Este arquivo contém instruções permanentes sobre a arquitetura do projeto, fluxo de atualização a partir do PhotoCraft original (upstream), versionamento semântico e conservação das customizações mobile. Você deve SEMPRE consultar este arquivo antes de efetuar qualquer alteração no repositório.

---

## 1. Identidade e Arquitetura do Projeto

- **Nome do Projeto**: PhotoCraft Android (mantido por Nepo / `@nepotira`).
- **Repositório**: [`nepotira/photocraft`](https://github.com/nepotira/photocraft) (Fork oficial de `storytold/photocraft`).
- **Linguagem**: **100% Rust** (Rust 2024, `wgpu`, `eframe`/`egui 0.36`, `winit`).
- **Isolamento de Código**:
  - `apps/photocraft-android/`: Contém todo o runner Android, empacotador (`Cargo.toml` para `cargo-apk`), ponte JNI nativa em Rust (`jni = "0.21"`), assistente de onboarding inicial (`onboarding.rs`), seletor de arquivos integrado (`file_picker.rs`) e keystore de assinatura (`release.keystore`).
  - `crates/ui-egui/`: Contém pequenas adaptações cirúrgicas para mobile (gestos de pinça e pan com 2 dedos em `canvas.rs`, diálogos responsivos com scroll em `dialogs.rs`).

---

## 2. Como Atualizar Quando Sair Nova Versão do PhotoCraft Original (Upstream)

Quando a equipe original do PhotoCraft (`storytold/photocraft`) lançar uma nova versão (ex: `v0.8.3`, `v0.9.0`):

### ❓ A IA precisa reescrever todas as funções do zero?
**NÃO!** Graças à arquitetura modular do Cargo Workspace e ao Git:
- O upstream PhotoCraft **não possui** a pasta `apps/photocraft-android/`.
- Nossas funcionalidades estão encapsuladas e são preservadas automaticamente pelo Git.
- O Git realiza o merge automático de 99% das novidades do programa original sem gastar tokens nem reescrever código existente!

### 📋 Passo a Passo para a IA Executar a Atualização:

1. **Sincronizar com o Upstream**:
   ```bash
   git fetch https://github.com/storytold/photocraft.git main
   git merge FETCH_HEAD -m "chore(upstream): sync latest PhotoCraft updates"
   ```
2. **Preservar Arquivos Críticos de Android**:
   - Garantir que `apps/photocraft-android/release.keystore` permaneça intocada (é a chave criptográfica que permite aos usuários atualizar o app sem desinstalar).
   - Verificar se `crates/ui-egui/src/canvas.rs` e `crates/ui-egui/src/dialogs.rs` mantiveram os blocos de multi-touch e diálogos responsivos. Se houver conflito de merge, resolver mantendo o suporte mobile.
3. **Atualizar Metadados e Versionamento**:
   - Consultar [`VERSIONING.md`](VERSIONING.md).
   - Elevar a versão em `apps/photocraft-android/Cargo.toml` (ex: `0.1.2` -> `0.1.3` ou `0.2.0`).
   - Atualizar a linha `Base Upstream: vX.Y.Z` em `VERSIONING.md` e `CHANGELOG.md`.
4. **Registrar no Changelog**:
   - Criar a nova entrada no [`CHANGELOG.md`](CHANGELOG.md) descrevendo as novidades herdadas do upstream e as melhorias do Android.
5. **Comitar e Compilar via CI**:
   - Fazer commit semântico (`chore(sync): update upstream PhotoCraft to vX.Y.Z`).
   - Fazer `git push origin main`.
   - Acompanhar a compilação do APK pelo GitHub Actions via `gh run watch`.

---

## 3. Regras de Ouro Técnicas

1. **Assinatura de Pacote**: NUNCA delete ou regere a `release.keystore`. Ela é fixa e definitiva.
2. **DPI e Escala**: NUNCA utilize `ctx.set_pixels_per_point(valor)` em loop a cada frame. Isso causa stuttering/flicker destrutivo no Android. Utilize SEMPRE `ctx.set_zoom_factor(factor)`.
3. **Tela Cheia Imersiva**: Chame `set_immersive_sticky()` para manter barras de status e navegação do Android ocultas em modo imersivo.
4. **Gestos de Dois Dedos**: Mantenha a distinção no canvas entre toque único (desenho/ferramenta ativa) e multitoque com dois ou mais dedos (zoom por pinça e movimentação de tela livres, estilo IbisPaint X).
5. **Seletor de Arquivos**: O seletor de arquivos integrado em `file_picker.rs` deve sempre atender a `Services::file_dialog` para garantir que o usuário consiga abrir e exportar arquivos pelo armazenamento do celular.
