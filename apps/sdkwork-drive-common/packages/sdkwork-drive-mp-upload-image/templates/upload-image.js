// Reference native component for the Drive upload-image family
// (`@sdkwork/drive-mp-upload-image`). Copy this folder into the host
// miniprogramRoot, then wire `createMpDriveUploadImage` from the host's
// bundled runtime — the upload service is built once by the host service
// layer from its declared upload intent (DRIVE_SPEC.md §18.3) and must NOT
// be composed inline here.
//
//   const { createMpDriveUploadImage, mapSnapshotForTemplate } =
//     require("<host-runtime>/drive-mp-upload-image");
//
// The component keeps the controller off `data` (it is not serializable);
// `state` carries only the rendered snapshot.
Component({
  properties: {
    label: { type: String, value: "" },
    description: { type: String, value: "" },
    readOnly: { type: Boolean, value: false },
    maxFiles: { type: Number, value: 1 },
    // Identifier of the persisted entity that owns these images.
    appResourceId: { type: String, value: "" },
  },

  data: {
    state: { items: [], canAddMore: true, isUploading: false, values: [] },
  },

  lifetimes: {
    attached() {
      const host = getApp();
      this.binding = createMpDriveUploadImage({
        // Host-provided service factory; see the package README.
        service: host.services.driveUploadImageService,
        maxFiles: this.data.maxFiles,
        resolveAppResourceId: () => this.data.appResourceId || null,
        onUploaded: (values) => {
          this.triggerEvent("uploaded", { values: [...values] });
        },
      });
      this.unsubscribe = this.binding.subscribe(() => this.syncState());
      this.syncState();
    },
    detached() {
      if (this.unsubscribe) {
        this.unsubscribe();
      }
      if (this.binding) {
        this.binding.destroy();
      }
    },
  },

  observers: {
    appResourceId(value) {
      if (this.binding) {
        this.syncState();
      }
    },
  },

  methods: {
    syncState() {
      if (!this.binding) {
        return;
      }
      this.setData({
        state: mapSnapshotForTemplate(this.binding.getSnapshot(), {
          maxFiles: this.data.maxFiles,
          readOnly: this.data.readOnly,
        }),
      });
    },

    onDriveUploadImagePick() {
      this.binding.chooseAndUpload({
        appResourceId: this.data.appResourceId || null,
        count: this.data.maxFiles,
      });
    },

    onDriveUploadImageRemove(event) {
      const itemId = event.currentTarget.dataset.id;
      if (typeof itemId === "string") {
        this.binding.removeItem(itemId);
      }
    },
  },
});
