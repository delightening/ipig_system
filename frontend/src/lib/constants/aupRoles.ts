// AUP 第 8 節「可擔任角色」選項。
// value 寫入 users.aup_roles（後端原值）；key 對應 i18n `profile.roles.*`。
// 治理：由 admin / IACUC 指派（見 admin UserEditDialog），使用者端唯讀（見 ProfileSettingsPage）。
export interface AupRoleOption {
    value: string
    key: string
}

export const AUP_ROLE_OPTIONS: AupRoleOption[] = [
    { value: 'PI', key: 'PI' },
    { value: 'Co-PI', key: 'Co-PI' },
    { value: 'Experimenter', key: 'Experimenter' },
    { value: 'Experiment Staff', key: 'ExperimentStaff' },
    { value: 'Veterinarian', key: 'Veterinarian' },
    { value: 'Animal Care Staff', key: 'AnimalCareStaff' },
]
