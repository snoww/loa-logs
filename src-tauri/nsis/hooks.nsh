!define TASK_NAME "LOA_Logs_Auto_Start"

!macro NSIS_HOOK_PREINSTALL

!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Update auto start task action if exists
  nsExec::Exec `powershell -NoProfile -ExecutionPolicy Bypass -Command "\
    Set-ScheduledTask -TaskName '${TASK_NAME}' -Action @(               \
      New-ScheduledTaskAction -Execute '$INSTDIR\${MAINBINARYNAME}.exe' \
    )"`
!macroend

!macro NSIS_HOOK_PREUNINSTALL

!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete /REBOOTOK "$INSTDIR\WinDivert64.sys"

  ; Remove auto start task if not updating
  ${If} $UpdateMode <> 1
    nsExec::Exec 'schtasks /delete /tn "${TASK_NAME}" /f'
  ${EndIf}
!macroend