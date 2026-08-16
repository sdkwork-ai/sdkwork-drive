import {
  BookOpen,
  Bot,
  CircleHelp,
  GitBranch,
  MessageCircle,
  Package,
  Rocket,
  Stamp,
  UserRound,
  UsersRound,
  Video,
  type LucideIcon,
} from 'lucide-react';

export interface SpaceTypeMeta {
  value: string;
  label: string;
  shortLabel: string;
  labelKey: string;
  descriptionKey: string;
  icon: LucideIcon;
  bgClass: string;
  textClass: string;
  description: string;
  isSystem: boolean;
}

export const SPACE_TYPES: SpaceTypeMeta[] = [
  {
    value: 'personal',
    label: 'Personal',
    shortLabel: 'Personal',
    labelKey: 'spaceTypePersonalLabel',
    descriptionKey: 'spaceTypePersonalDesc',
    icon: UserRound,
    bgClass: 'bg-blue-50 dark:bg-blue-950/30',
    textClass: 'text-blue-700 dark:text-blue-300',
    description: 'User personal files and documents',
    isSystem: false,
  },
  {
    value: 'team',
    label: 'Team',
    shortLabel: 'Team',
    labelKey: 'spaceTypeTeamLabel',
    descriptionKey: 'spaceTypeTeamDesc',
    icon: UsersRound,
    bgClass: 'bg-purple-50 dark:bg-purple-950/30',
    textClass: 'text-purple-700 dark:text-purple-300',
    description: 'Team shared workspace for collaboration',
    isSystem: false,
  },
  {
    value: 'knowledge_base',
    label: 'Knowledge Base',
    shortLabel: 'KB',
    labelKey: 'spaceTypeKnowledgeBaseLabel',
    descriptionKey: 'spaceTypeKnowledgeBaseDesc',
    icon: BookOpen,
    bgClass: 'bg-emerald-50 dark:bg-emerald-950/30',
    textClass: 'text-emerald-700 dark:text-emerald-300',
    description: 'Knowledge base articles and resources',
    isSystem: false,
  },
  {
    value: 'ai_generated',
    label: 'AI Generated',
    shortLabel: 'AI',
    labelKey: 'spaceTypeAiGeneratedLabel',
    descriptionKey: 'spaceTypeAiGeneratedDesc',
    icon: Bot,
    bgClass: 'bg-cyan-50 dark:bg-cyan-950/30',
    textClass: 'text-cyan-700 dark:text-cyan-300',
    description: 'AI-generated content and outputs',
    isSystem: false,
  },
  {
    value: 'git_repository',
    label: 'Git Repository',
    shortLabel: 'Git',
    labelKey: 'spaceTypeGitRepositoryLabel',
    descriptionKey: 'spaceTypeGitRepositoryDesc',
    icon: GitBranch,
    bgClass: 'bg-orange-50 dark:bg-orange-950/30',
    textClass: 'text-orange-700 dark:text-orange-300',
    description: 'Git repository storage for version control',
    isSystem: true,
  },
  {
    value: 'deployment',
    label: 'Deployment',
    shortLabel: 'Deploy',
    labelKey: 'spaceTypeDeploymentLabel',
    descriptionKey: 'spaceTypeDeploymentDesc',
    icon: Rocket,
    bgClass: 'bg-indigo-50 dark:bg-indigo-950/30',
    textClass: 'text-indigo-700 dark:text-indigo-300',
    description: 'Deployment artifacts and releases',
    isSystem: true,
  },
  {
    value: 'app_upload',
    label: 'App Upload',
    shortLabel: 'App',
    labelKey: 'spaceTypeAppUploadLabel',
    descriptionKey: 'spaceTypeAppUploadDesc',
    icon: Package,
    bgClass: 'bg-pink-50 dark:bg-pink-950/30',
    textClass: 'text-pink-700 dark:text-pink-300',
    description: 'Application upload storage for SDKWork apps',
    isSystem: true,
  },
  {
    value: 'im',
    label: 'IM',
    shortLabel: 'IM',
    labelKey: 'spaceTypeImLabel',
    descriptionKey: 'spaceTypeImDesc',
    icon: MessageCircle,
    bgClass: 'bg-teal-50 dark:bg-teal-950/30',
    textClass: 'text-teal-700 dark:text-teal-300',
    description: 'Instant messaging attachments and media',
    isSystem: true,
  },
  {
    value: 'rtc',
    label: 'RTC',
    shortLabel: 'RTC',
    labelKey: 'spaceTypeRtcLabel',
    descriptionKey: 'spaceTypeRtcDesc',
    icon: Video,
    bgClass: 'bg-red-50 dark:bg-red-950/30',
    textClass: 'text-red-700 dark:text-red-300',
    description: 'Real-time communication recordings and files',
    isSystem: true,
  },
  {
    value: 'notary',
    label: 'Notary',
    shortLabel: 'Notary',
    labelKey: 'spaceTypeNotaryLabel',
    descriptionKey: 'spaceTypeNotaryDesc',
    icon: Stamp,
    bgClass: 'bg-amber-50 dark:bg-amber-950/30',
    textClass: 'text-amber-700 dark:text-amber-300',
    description: 'Notary documents and attestations',
    isSystem: true,
  },
];

export function getSpaceTypeMeta(type: string): SpaceTypeMeta {
  return SPACE_TYPES.find((entry) => entry.value === type) ?? {
    value: type,
    label: type,
    shortLabel: type.slice(0, 6),
    labelKey: '',
    descriptionKey: '',
    icon: CircleHelp,
    bgClass: 'bg-neutral-50 dark:bg-neutral-800',
    textClass: 'text-neutral-700 dark:text-neutral-300',
    description: '',
    isSystem: false,
  };
}

export function resolveSpaceTypeLabel(
  meta: SpaceTypeMeta,
  t: (key: string) => string,
): string {
  return meta.labelKey ? t(meta.labelKey) : meta.label;
}

export function resolveSpaceTypeDescription(
  meta: SpaceTypeMeta,
  t: (key: string) => string,
): string {
  return meta.descriptionKey ? t(meta.descriptionKey) : meta.description;
}
