# 📐 Regras de Versionamento Semântico e Documentação - PhotoCraft Android

Este documento estabelece as diretrizes obrigatórias de versionamento, versionamento semântico (SemVer 2.0), rastreamento de erros e controle de versões do **PhotoCraft Android** (mantido por Nepo / `@nepotira`).

> **REGRA FUNDAMENTAL PARA O ASSISTENTE IA:**
> Antes de qualquer commit, modificação de código ou geração de APK, você DEVE consultar este arquivo, incrementar a versão semântica e o `version_code` de acordo com as alterações, e registrar todos os erros relatados e suas respectivas soluções no `CHANGELOG.md`.

---

## 1. Esquema de Versionamento

Cada lançamento do PhotoCraft Android possui duas referências de versão:

### 1.1 Versão Semântica (SemVer)
Formato: `v<MAJOR>.<MINOR>.<PATCH>-android`

- **MAJOR (`X.0.0`)**: 
  - Quebra de compatibilidade severa (ex: alteração radical de arquitetura, incompatibilidade com versões antigas do Android ou migração de engine).
- **MINOR (`0.X.0`)**:
  - Novas funcionalidades voltadas para o ecossistema Android (ex: novo assistente de onboarding, suporte a Samsung DeX, arrastar e soltar de arquivos, atalhos de caneta/touch, novos módulos de exportação).
- **PATCH (`0.0.X`)**:
  - Correções de bugs, ajustes de escala/DPI, correções de renderização, hotfixes de permissões, ajustes de fullscreen e estabilidade.
- **Sufixo `-android`**:
  - Identifica explicitamente o port Android customizado.
- **Rastreamento Upstream**:
  - Deve sempre ser documentada a versão base do PhotoCraft original da storytold (ex: `Base Upstream: v0.17.0`).

### 1.2 Android `version_code` (Obrigatório)
No Android, o instalador do sistema (`PackageInstaller` / Google Play) utiliza um número inteiro sequencial para determinar se um APK é uma atualização válida:

- **Regra**: O `version_code` em `apps/photocraft-android/Cargo.toml` **DEVE ser incrementado em +1 a cada nova compilação / release**.
- **Nunca reutilizar um `version_code` anterior**, pois isso impede atualizações diretas no aparelho do usuário.

---

## 2. Assinatura Criptográfica e Integridade do Pacote

- **Keystore Permanente**: O arquivo de keystore `apps/photocraft-android/release.keystore` deve ser fixo e persistido no repositório.
- **Proibido gerar novas chaves a cada build**: Gerar uma nova chave em tempo de CI altera o hash criptográfico do certificado, fazendo com que o Android rejeite a atualização com o erro: *"Como o pacote parece ser inválido, o app não foi instalado"*.
- **Alias e Senha**:
  - Alias: `androidkey`
  - Senha: `android`

---

## 3. Fluxo de Documentação de Erros (Bug Tracking)

Sempre que o usuário relatar erros ou comportamentos inesperados no chat, o assistente deve:

1. **Catalogar o Erro**:
   - Atribuir um identificador: `[BUG-XXX]`.
   - Registrar o sintoma relatado pelo usuário.
   - Diagnosticar a causa raiz técnica no código ou no Android.
2. **Aplicar a Correção**:
   - Implementar a solução técnica no código.
   - Atribuir a identificação `[FIX-XXX]`.
3. **Atualizar o `CHANGELOG.md`**:
   - Registrar na versão apropriada o que falhou na versão anterior e como foi corrigido na nova.
4. **Incrementar Versão**:
   - Elevar `PATCH` (ou `MINOR` se novas funções foram adicionadas) e incrementar `version_code += 1`.

---

## 4. Estrutura do `Cargo.toml` para Cada Release

O empacotador oficial `cargo-apk` deriva automaticamente o `versionCode` numérico e o `versionName` a partir do campo nativo `version` do Cargo sob `[package]`. **Não** declare `version_name` ou `version_code` sob `[package.metadata.android]`, pois o `cargo-apk` rejeita chaves manuais com pânico.

Ao lançar uma nova versão, sincronize o campo `version` em `apps/photocraft-android/Cargo.toml`:

```toml
[package]
name = "photocraft-android"
version = "<MAJOR>.<MINOR>.<PATCH>" # Ex: "0.1.1"

[package.metadata.android]
package = "ai.storyteller.photocraft"
apk_label = "PhotoCraft"
fullscreen = true
resizeable_activity = true
```
