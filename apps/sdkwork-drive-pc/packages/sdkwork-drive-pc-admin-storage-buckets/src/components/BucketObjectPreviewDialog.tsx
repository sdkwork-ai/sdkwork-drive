import React, { useMemo, useState } from 'react';
import { ConfirmDialog, Modal, ModalContent, ModalHeader, ModalTitle } from '@sdkwork/ui-pc-react';
import {
  FilePreviewSurface,
  formatFilePreviewLabel,
  useFilePreviewLabels,
  type FilePreviewLabelsInput,
  type FilePreviewNavigation,
  type FilePreviewResource,
} from 'sdkwork-drive-pc-file-preview';
import type {
  StorageProviderAdminService,
  StorageProviderObjectView,
  StorageProviderView,
} from 'sdkwork-drive-pc-admin-storage-providers';
import {
  MAX_OBJECT_CONTENT_BYTES,
  downloadObjectBytes,
  readObjectBytes,
  readObjectText,
  writeObjectText,
} from '../utils/objectContent';
import { useUnsavedChangesGuard } from '../hooks/useUnsavedChangesGuard';

export interface BucketObjectPreviewDialogProps {
  bucket: string;
  /** 预览文案：只需给出翻译过的字段，其余由预览包的英文兜底补齐。 */
  labels?: FilePreviewLabelsInput;
  object: StorageProviderObjectView;
  /** 相邻文件切换：由调用方（它才知道当前目录与筛选）给出位置与回调。 */
  navigation?: FilePreviewNavigation;
  onClose: () => void;
  /** 内容被保存后的通知；宿主据此刷新列表里该对象的体积与修改时间。 */
  onSaved?: () => void;
  provider: StorageProviderView;
  /** 桶所在地域：读取内容要按这个桶自己的地域找端点。 */
  region?: string;
  service: StorageProviderAdminService;
}

/**
 * 存储桶对象预览/编辑对话框。
 *
 * 它是「后端内容接口」与「可复用预览组件」之间的适配器：把 base64 内容接口折成
 * `FilePreviewResource` 端口（读字节 / 读文本 / 写回 / 下载），预览组件本身完全不知道
 * 对象存储、桶名或鉴权。这样同一批预览组件也能挂在 drive 控制台的预签名直链上。
 */
export function BucketObjectPreviewDialog({
  bucket,
  labels,
  navigation,
  object,
  onClose,
  onSaved,
  provider,
  region,
  service,
}: BucketObjectPreviewDialogProps) {
  const mergedLabels = useFilePreviewLabels(labels);
  const [dirty, setDirty] = useState(false);
  const guard = useUnsavedChangesGuard({ dirty, onClose });
  const resource = useMemo<FilePreviewResource>(() => {
    const scope = { bucket, providerId: provider.id, region, service };
    const name = object.key.split('/').filter(Boolean).at(-1) ?? object.key;
    const writable = !object.isFolder;
    return {
      name,
      contentType: object.contentType,
      sizeBytes: object.sizeBytes,
      inlineLimitBytes: MAX_OBJECT_CONTENT_BYTES,
      readBytes: (options) => readObjectBytes(scope, object.key, options?.signal),
      readText: (options) => readObjectText(scope, object.key, options?.signal),
      saveText: writable
        ? (text) => writeObjectText(scope, object.key, text, object.contentType)
        : undefined,
      download: () => {
        void downloadObjectBytes(scope, object);
      },
    };
  }, [bucket, object, provider.id, region, service]);

  return (
    <>
      <Modal open onOpenChange={(next) => (next ? undefined : guard.requestClose())}>
        {/*
          预览铺满整个视口。
          
          - `inset-0 + h-auto/w-auto` 覆盖库里的定宽定高与 `left-1/2/top-1/2` 定位；
            `translate-x-0/translate-y-0` 必须显式写出来，否则库里那对 `-translate-1/2`
            会把已经铺满的元素整体推出去半个屏幕。
          - 不用 `w-[100vw]`：视口有纵向滚动条时 100vw 会比可用宽度大，反而多出一条横向滚动条。
          - `rounded-none`：贴边时圆角只会被视口裁掉一半，看着像缺角。
        */}
        <ModalContent
          align="center"
          className="inset-0 h-auto w-auto max-h-none max-w-none translate-x-0 translate-y-0 grid-rows-[auto_minmax(0,1fr)] overflow-hidden rounded-none"
          size="full"
          /*
            关掉库里那个浮在右上角的 X。
            
            它是 `absolute right-4 top-4`，与文件浏览器自己的关闭按钮落在同一个位置：
            两个 X 叠在一起，用户点哪个都心里没底（而且它压在预览内容之上）。预览工具条里
            本来就有自己的关闭按钮，这里只保留那一个；Esc 也仍然能关闭。
          */
          showCloseButton={false}
        >
          {/* 可见的标题在预览工具条里（带文件名与类型徽标），这里只保留无障碍名称。 */}
          <ModalHeader className="sr-only">
            <ModalTitle>
              {formatFilePreviewLabel(mergedLabels.preview, { name: resource.name })}
            </ModalTitle>
          </ModalHeader>
          <div className="flex min-h-0 flex-col overflow-hidden">
            <FilePreviewSurface
              labels={mergedLabels}
              navigation={navigation}
              onClose={guard.requestClose}
              onDirtyChange={setDirty}
              onSaved={onSaved}
              resource={resource}
            />
          </div>
        </ModalContent>
      </Modal>

      {/*
        关闭前确认：草稿只在内存里，一次误触就会丢掉刚编辑的内容。对话框自带的 X、
        Esc 与工具条里的关闭按钮都汇到同一个守卫上。
      */}
      <ConfirmDialog
        cancelLabel={mergedLabels.cancel}
        confirmLabel={mergedLabels.discardChanges}
        description={mergedLabels.unsavedCloseConfirm}
        onConfirm={guard.confirmDiscard}
        onOpenChange={(next) => {
          if (!next) {
            guard.cancelDiscard();
          }
        }}
        open={guard.pending}
        title={mergedLabels.unsavedChanges}
        tone="danger"
      />
    </>
  );
}
