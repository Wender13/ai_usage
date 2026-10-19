import type { Messages } from "./en";

const pt: Messages = {
  app: {
    lastCheck: "Última consulta às {time}",
    waiting: "Aguardando a primeira consulta…",
    refresh: "Atualizar",
    refreshing: "Atualizando…",
  },
  sections: {
    rules: "Regras",
    settings: "Configurações",
    overlay: "Overlay",
  },
  providers: {
    claude: "Claude",
  },
  windows: {
    rolling_5h: "Sessão (5h)",
    rolling_7d: "Semanal (7 dias)",
  },
  windowsShort: {
    rolling_5h: "5h",
    rolling_7d: "7d",
  },
  status: {
    loading: "Carregando…",
    stale: "Valores da última leitura bem-sucedida, às {time}.",
  },
  time: {
    lessThanMinute: "menos de 1 min",
    resetUnknown: "horário de reset desconhecido",
    resetting: "reiniciando…",
    resetsIn: "reinicia em {duration} ({when})",
  },
  rules: {
    new: "Nova regra",
    empty: "Nenhuma regra cadastrada.",
    enable: "Ativar regra",
    disable: "Desativar regra",
    edit: "Editar",
    delete: "Excluir",
    anyProvider: "qualquer provider",
    anyWindow: "qualquer janela",
    types: {
      threshold: "Limiar de uso",
      interval: "Resumo periódico",
      beforeReset: "Antes do reset",
    },
    channels: {
      notification: "Notificação",
      overlay: "Overlay",
      both: "Notificação e overlay",
    },
    describe: {
      threshold: "Ao atingir {percent}% em {target}",
      interval: "Resumo do uso a cada {minutes} min",
      beforeReset: "{minutes} min antes do reset de {target}",
    },
    form: {
      type: "Tipo",
      provider: "Provider",
      window: "Janela",
      any: "Qualquer",
      threshold: "Limiar (%)",
      every: "A cada (min)",
      minutesBefore: "Minutos antes do reset",
      channel: "Canal",
      hint: "Verificada a cada consulta de uso, então a precisão é a do intervalo de consulta.",
      cancel: "Cancelar",
      save: "Salvar",
      create: "Criar regra",
    },
  },
  settings: {
    checkEvery: "Consultar o uso a cada",
    minutes: "min",
    save: "Salvar",
    saved: "Salvo.",
    language: "Idioma",
    languageAuto: "Automático (sistema)",
  },
  overlay: {
    show: "Mostrar overlay",
    position: "Posição",
    custom: "Personalizada (arrastada)",
    opacity: "Opacidade do fundo: {value}%",
    lock: "Travar posição (impede arrastar o overlay)",
    fieldsLegend: "Informações exibidas",
    fields: {
      showProvider: "Nome do provider",
      showLabel: "Nome da janela (5h, 7d)",
      showPercent: "Percentual",
      showReset: "Tempo até o reset",
    },
    windowsLegend: "Janelas exibidas",
    xwayland: "Usar XWayland",
    xwaylandHint:
      'No GNOME em Wayland, posição, trava e "sempre no topo" só funcionam via XWayland. Vale depois de reiniciar o app.',
    dismiss: "Clique para dispensar",
    moreAlerts: "+{count} alerta(s)",
    lastCheckFailed: "Erro na última consulta",
  },
  anchors: {
    top_left: "Superior esquerdo",
    top_center: "Superior central",
    top_right: "Superior direito",
    bottom_left: "Inferior esquerdo",
    bottom_center: "Inferior central",
    bottom_right: "Inferior direito",
  },
};

export default pt;
