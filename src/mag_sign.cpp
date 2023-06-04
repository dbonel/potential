#include "plugin.hpp"

struct MagSign : Module {
    enum ParamId { PARAMS_LEN };
    enum InputId { BIPOLAR_INPUT, MAGNITUDE_INPUT, SIGN_INPUT, INPUTS_LEN };
    enum OutputId {
        MAGNITUDE_OUTPUT,
        SIGN_OUTPUT,
        BIPOLAR_OUTPUT,
        OUTPUTS_LEN
    };
    enum LightId { LIGHTS_LEN };

    MagSign() {
        config(PARAMS_LEN, INPUTS_LEN, OUTPUTS_LEN, LIGHTS_LEN);
        configInput(BIPOLAR_INPUT, "Bipolar");
        configInput(MAGNITUDE_INPUT, "Magnitude");
        configInput(SIGN_INPUT, "Sign");
        configOutput(MAGNITUDE_OUTPUT, "Magnitude");
        configOutput(SIGN_OUTPUT, "Sign");
        configOutput(BIPOLAR_OUTPUT, "Bipolar");
    }

    void process(const ProcessArgs &args) override {
        const rustlib::Port *inputs = ffi_port(&this->inputs[0]);
        rustlib::Port *outputs = ffi_port(&this->outputs[0]);
        rustlib::mag_sign_process_raw(inputs, outputs);
    }
};

struct MagSignWidget : ModuleWidget {
    MagSignWidget(MagSign *module) {
        setModule(module);
        setPanel(createPanel(asset::plugin(pluginInstance, "res/MagSign.svg")));

        addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
        addChild(createWidget<ScrewSilver>(
            Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
        addChild(
            createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH,
                                          RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

        addInput(createInputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 21.508)), module, MagSign::BIPOLAR_INPUT));
        addInput(createInputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 81.372)), module, MagSign::MAGNITUDE_INPUT));
        addInput(createInputCentered<PJ301MPort>(mm2px(Vec(7.62, 97.367)),
                                                 module, MagSign::SIGN_INPUT));

        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 37.504)), module, MagSign::MAGNITUDE_OUTPUT));
        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 53.5)), module, MagSign::SIGN_OUTPUT));
        addOutput(createOutputCentered<PJ301MPort>(
            mm2px(Vec(7.62, 113.363)), module, MagSign::BIPOLAR_OUTPUT));
    }
};

Model *modelMagSign = createModel<MagSign, MagSignWidget>("MagSign");
