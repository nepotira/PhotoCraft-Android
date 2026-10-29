# 🔄 Regra Universal de Sincronização e Rastreamento Contínuo do Upstream
# Repositório: PhotoCraft Android (nepotira/PhotoCraft-Android)
# Alvo: storytold/photocraft (Upstream Oficial)

---

## 📌 POLÍTICA OBRIGATÓRIA (URGENTE & PERMANENTE)

Todo agente de IA (Antigravity ou outro) e desenvolvedor trabalhando neste repositório DEVE, **sem exceção e no início de qualquer ciclo de trabalho**, verificar e incorporar a versão base mais recente lançada pelo PhotoCraft original (`storytold/photocraft`).

### 1. Protocolo de Verificação Obrigatória (Always Check)
A cada inicialização de tarefa ou antes de efetuar commits de melhorias:

1. **Buscar tags e commits upstream**:
   ```bash
   git fetch upstream --tags
   ```
2. **Comparar a versão base atual**:
   - Inspecione a versão base registrada em [`VERSIONING.md`](../../VERSIONING.md) (ex: `Base Upstream: v0.8.2`).
   - Identifique a tag estável mais recente do upstream:
     ```bash
     git tag -l "v*" --sort=-v:refname | head -n 1
     ```
3. **Se houver nova versão ou novos commits**:
   - O agente DEVE incorporar a atualização upstream antes de fechar o ciclo de entrega.

---

## 2. Procedimento de Merge e Preservação de Customizações Mobile

O upstream oficial do PhotoCraft não possui a pasta `apps/photocraft-android/`. O isolamento modular do Cargo Workspace garante que as funcionalidades Android permaneçam protegidas durante o merge.

### 2.1 Execução do Merge:
```bash
git merge upstream/main -m "chore(upstream): sync latest PhotoCraft updates (base vX.Y.Z)"
```
*(ou apontando para a tag específica lançada, ex: `v0.17.0`)*

### 2.2 Arquivos Críticos com Proteção Absoluta contra Conflitos:
1. **`apps/photocraft-android/release.keystore`**:
   - NUNCA substituir ou apagar. É a chave RSA persistente da assinatura do APK.
2. **`crates/ui-egui/src/canvas.rs`**:
   - Manter cirurgicamente o bloco de detecção multitoque:
     - `multi_touch.num_touches >= 2`: pinça de zoom, rotação angular e pan do canvas.
     - Supressão de strokes da ferramenta ativa durante gestos de dois dedos.
3. **`crates/ui-egui/src/dialogs.rs`**:
   - Manter a responsividade dinâmica de largura baseada em `ctx.content_rect().width()` e o encapsulamento em `egui::ScrollArea::both()`.
4. **`apps/photocraft-android/`**:
   - Todo o runner Android, JNI, Onboarding e File Picker integrado.

---

## 3. Versionamento Semântico e Metadados Pós-Sincronização

Após a sincronização upstream bem-sucedida:
1. Atualize a linha `Base Upstream: vX.Y.Z` em:
   - [`VERSIONING.md`](../../VERSIONING.md)
   - [`CHANGELOG.md`](../../CHANGELOG.md)
   - [`README.md`](../../README.md)
2. Incremente o `version_code` em `apps/photocraft-android/Cargo.toml` (`+1`).
3. Registre no [`CHANGELOG.md`](../../CHANGELOG.md) as novidades herdadas do upstream e as adaptações mobile correspondentes.
