<!--
Obrigado por contribuir com o Local App.

Baseie o PR em `main`, a única branch de integração. Apague as seções que não se aplicam.
-->

## O que muda

<!-- Qual comportamento é diferente depois, e por quê. Se fecha uma issue, escreva `Fixes #123`. -->

## Como você verificou

<!-- Os comandos que rodou e o que disseram. -->

- [ ] `cargo xtask fmt`
- [ ] clippy com `-D warnings` (`cargo clippy --locked --workspace --all-targets -- -D warnings`)
- [ ] `cargo test --locked`
- [ ] `cargo xtask layers` (se mexi em fronteira de crate) e `cargo xtask assets` (se adicionei asset)

**Testes.** Comportamento novo precisa de teste na escala certa — unitário, de integração ou ponta a
ponta. Para correção de bug, **confirme que o teste falha antes da correção**.

## Never-crash

Todo arquivo, caminho e entrada do usuário é entrada não confiável (`AGENTS.md`).

- [ ] Nenhum `unwrap`/`expect`/`panic!`/`unreachable!`/`todo!` novo fora de testes
- [ ] Nenhum `unsafe` novo
- [ ] Todo crash corrigido tem um teste sintético de regressão

## Proveniência

- [ ] Nenhum asset copiado de produto proprietário (Adobe e afins)
- [ ] Nenhum arquivo de fonte commitado aqui (eles vivem em
      [storytold/craft-fonts](https://github.com/storytold/craft-fonts))
- [ ] Todo asset novo tem linha em `assets/ATTRIBUTION.md`
- [ ] Nenhuma dependência `git+` (o CI valida)

## Documentação

- [ ] Docs atualizados se o comportamento documentado mudou (`docs/`, README)

## O que o revisor deve olhar primeiro

<!-- A decisão que você tem menos certeza, uma restrição deliberada que quer fixada, ou uma
     mudança de comportamento que quem já usa vai notar. -->
